<?php
/**
 * MyRoots_Store: where the data of the engine lives inside WordPress.
 *
 * - Records (people, unions, pets, sources, media metadata, places, settings…)
 *   are stored as the engine writes them, one JSON document per row of
 *   {prefix}myroots_records. The engine owns the format; WordPress only keeps it.
 * - Photo, scan and recording files: wp-content/uploads/myroots/files-<secret>/
 *   (served only through the REST API to editors).
 * - The published family website: wp-content/uploads/myroots/site-<secret>/
 *   (served by MyRoots_Site, so the visibility setting is respected).
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Store {

	const DB_VERSION = 1;
	const KINDS      = [ 'person', 'union', 'pet', 'source', 'media', 'place', 'revision', 'suggestion', 'settings', 'meta' ];

	/** Records table of a tree (the first tree keeps the name of MyRoots 3.0). */
	public static function table( ?string $tree = null ): string {
		global $wpdb;
		return $wpdb->prefix . 'myroots_records' . MyRoots_Trees::suffix( $tree );
	}

	public static function install(): void {
		self::install_tree( MyRoots_Trees::DEF );
		update_option( 'myroots3_db', self::DB_VERSION, false );
		if ( ! get_option( 'myroots3_secret' ) ) {
			update_option( 'myroots3_secret', wp_generate_password( 20, false, false ), false );
		}
		if ( ! get_option( 'myroots3_gen' ) ) update_option( 'myroots3_gen', 1, false );
		self::dir( 'files' );
		self::dir( 'site' );
	}

	/** The table of one tree. */
	public static function install_tree( string $tree ): void {
		global $wpdb;
		require_once ABSPATH . 'wp-admin/includes/upgrade.php';
		$charset = $wpdb->get_charset_collate();
		$table   = self::table( $tree );
		dbDelta( "CREATE TABLE {$table} (
			kind varchar(20) NOT NULL,
			rid bigint(20) unsigned NOT NULL,
			data longtext NOT NULL,
			updated datetime DEFAULT NULL,
			PRIMARY KEY  (kind,rid)
		) {$charset};" );
		if ( $tree !== MyRoots_Trees::DEF && ! get_option( MyRoots_Trees::opt_name( 'myroots3_gen', $tree ) ) ) {
			update_option( MyRoots_Trees::opt_name( 'myroots3_gen', $tree ), 1, false );
		}
	}

	/** Removes the table and the folders of a tree. */
	public static function drop_tree( string $tree ): void {
		global $wpdb;
		$table = self::table( $tree );
		$wpdb->query( $wpdb->prepare( 'DROP TABLE IF EXISTS %i', $table ) ); // phpcs:ignore WordPress.DB.DirectDatabaseQuery, WordPress.DB.DirectDatabaseQuery.SchemaChange -- own table of a deleted tree.
		foreach ( [ 'files', 'site' ] as $what ) self::rmdir( self::dir( $what, $tree, false ) );
	}

	private static function rmdir( string $dir ): void {
		if ( ! is_dir( $dir ) ) return;
		$it = new RecursiveIteratorIterator( new RecursiveDirectoryIterator( $dir, FilesystemIterator::SKIP_DOTS ), RecursiveIteratorIterator::CHILD_FIRST );
		foreach ( $it as $f ) {
			if ( $f->isDir() ) rmdir( $f->getPathname() ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_rmdir
			else wp_delete_file( $f->getPathname() );
		}
		rmdir( $dir ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_rmdir
	}

	public static function maybe_upgrade(): void {
		if ( (int) get_option( 'myroots3_db' ) !== self::DB_VERSION ) self::install();
	}

	/**
	 * A private folder in uploads (created on demand, closed for direct access
	 * on Apache; on other servers the random name keeps it unguessable).
	 */
	public static function dir( string $what, ?string $tree = null, bool $create = true ): string {
		$up   = wp_upload_dir( null, false );
		$base = trailingslashit( $up['basedir'] ) . 'myroots';
		$dir  = $base . '/' . $what . '-' . get_option( 'myroots3_secret', 'x' ) . str_replace( '_', '-', MyRoots_Trees::suffix( $tree ) );
		if ( $create && ! is_dir( $dir ) && wp_mkdir_p( $dir ) ) {
			foreach ( [ $base, $dir ] as $d ) {
				if ( ! wp_is_writable( $d ) ) continue;
				// phpcs:disable WordPress.WP.AlternativeFunctions.file_system_operations_file_put_contents
				if ( ! file_exists( $d . '/index.php' ) ) file_put_contents( $d . '/index.php', "<?php // Silence is golden.\n" );
				if ( ! file_exists( $d . '/.htaccess' ) ) file_put_contents( $d . '/.htaccess', "Require all denied\nDeny from all\n" );
				// phpcs:enable
			}
		}
		return $dir;
	}

	/** Can the plugin store files (photos, the website)? */
	public static function writable(): bool {
		$up = wp_upload_dir( null, false );
		return empty( $up['error'] ) && wp_is_writable( $up['basedir'] );
	}

	/* ------------------------------------------------------------------ */
	/*  Records                                                            */
	/* ------------------------------------------------------------------ */

	/**
	 * All records, as the engine starts from them. Decoded as objects so empty
	 * JSON objects stay objects when the REST API encodes them again.
	 */
	/**
	 * Sends all records as a JSON array, a page of rows at a time and without
	 * decoding them: big trees fit in the memory limit of any hosting.
	 */
	public static function stream_all(): void {
		global $wpdb;
		$table = self::table();
		while ( ob_get_level() ) ob_end_clean();
		header( 'Content-Type: application/json; charset=UTF-8' );
		header( 'Cache-Control: no-store' );
		header( 'X-Content-Type-Options: nosniff' );
		// JSON shrinks about ten times: compressed here when the server does not do it itself
		if ( extension_loaded( 'zlib' ) && ! ini_get( 'zlib.output_compression' ) ) ob_start( 'ob_gzhandler' );
		echo '[';
		$first = true;
		$check = function_exists( 'json_validate' );
		$kind  = '';
		$rid   = -1;
		do {
			// phpcs:ignore WordPress.DB.DirectDatabaseQuery -- own table of the tree, read once per editor start, in pages.
			$rows = (array) $wpdb->get_results( $wpdb->prepare( 'SELECT kind, rid, data FROM %i WHERE kind > %s OR (kind = %s AND rid > %d) ORDER BY kind, rid LIMIT 2000', $table, $kind, $kind, $rid ) );
			foreach ( $rows as $r ) {
				$kind = (string) $r->kind;
				$rid  = (int) $r->rid;
				$d    = trim( (string) $r->data );
				if ( $d === '' || $d[0] !== '{' || ( $check && ! json_validate( $d ) ) ) continue;
				echo $first ? '' : ',', $d; // phpcs:ignore WordPress.Security.EscapeOutput.OutputNotEscaped -- JSON written by the app itself, sent as JSON.
				$first = false;
			}
		} while ( count( $rows ) === 2000 );
		echo ']';
		exit;
	}

	public static function all(): array {
		global $wpdb;
		$table = self::table();
		// phpcs:ignore WordPress.DB.DirectDatabaseQuery -- own table of the tree, read once per editor start; no cache, every edit changes it.
		$rows = (array) $wpdb->get_col( $wpdb->prepare( 'SELECT data FROM %i', $table ) );
		$out  = [];
		foreach ( $rows as $json ) {
			$r = json_decode( (string) $json, false );
			if ( is_object( $r ) ) $out[] = $r;
		}
		return $out;
	}

	public static function count( string $kind ): int {
		global $wpdb;
		$table = self::table();
		// phpcs:ignore WordPress.DB.DirectDatabaseQuery -- own table of the tree, always current.
		return (int) $wpdb->get_var( $wpdb->prepare( 'SELECT COUNT(*) FROM %i WHERE kind = %s', $table, $kind ) );
	}

	public static function get( string $kind, int $id ): ?array {
		global $wpdb;
		$table = self::table();
		// phpcs:ignore WordPress.DB.DirectDatabaseQuery -- own table of the tree, always current.
		$json = $wpdb->get_var( $wpdb->prepare( 'SELECT data FROM %i WHERE kind = %s AND rid = %d', $table, $kind, $id ) );
		if ( ! $json ) return null;
		$r = json_decode( $json, true );
		return is_array( $r ) ? $r : null;
	}

	/**
	 * Applies a change set of the engine: [{kind, id, json}], json === null deletes.
	 *
	 * @return int|WP_Error number of rows written
	 */
	public static function apply( array $records, array $deleted_media ) {
		global $wpdb;
		$table = self::table();
		$now   = current_time( 'mysql', true );
		$n     = 0;
		// phpcs:disable WordPress.DB.DirectDatabaseQuery -- own table; the engine's change set is written in one transaction.
		$wpdb->query( 'START TRANSACTION' );
		foreach ( $records as $r ) {
			if ( ! is_array( $r ) ) continue;
			$kind = isset( $r['kind'] ) ? (string) $r['kind'] : '';
			$id   = isset( $r['id'] ) ? (int) $r['id'] : -1;
			if ( ! in_array( $kind, self::KINDS, true ) || $id < 0 ) continue;
			// The record travels as the exact JSON text of the engine: decoding and
			// encoding it in PHP would turn empty objects into arrays.
			$json = isset( $r['json'] ) && is_string( $r['json'] ) ? $r['json'] : null;
			if ( $json === null ) {
				$wpdb->delete( $table, [ 'kind' => $kind, 'rid' => $id ], [ '%s', '%d' ] );
			} else {
				$check = json_decode( $json, true );
				if ( ! is_array( $check ) || ( $check['kind'] ?? '' ) !== $kind || (int) ( $check['id'] ?? -1 ) !== $id || ! isset( $check['data'] ) ) continue;
				$ok = $wpdb->replace( $table, [ 'kind' => $kind, 'rid' => $id, 'data' => $json, 'updated' => $now ], [ '%s', '%d', '%s', '%s' ] );
				if ( $ok === false ) {
					$wpdb->query( 'ROLLBACK' );
					return new WP_Error( 'myroots_db', $wpdb->last_error ?: 'database error', [ 'status' => 500 ] );
				}
			}
			$n++;
		}
		$wpdb->query( 'COMMIT' );
		// phpcs:enable
		foreach ( $deleted_media as $mid ) self::delete_media( (int) $mid );
		if ( $n ) MyRoots_Trees::update_opt( 'myroots3_changed', time() );
		return $n;
	}

	/** Removes everything (restoring a .myroots backup replaces the tree). */
	public static function reset(): int {
		global $wpdb;
		$table = self::table();
		// phpcs:ignore WordPress.DB.DirectDatabaseQuery -- own table of the tree, emptied before a backup is restored.
		$wpdb->query( $wpdb->prepare( 'DELETE FROM %i', $table ) );
		foreach ( (array) glob( self::dir( 'files' ) . '/*.bin' ) as $f ) {
			if ( is_string( $f ) ) wp_delete_file( $f );
		}
		$gen = (int) MyRoots_Trees::get_opt( 'myroots3_gen', 1 ) + 1;
		MyRoots_Trees::update_opt( 'myroots3_gen', $gen );
		MyRoots_Trees::update_opt( 'myroots3_changed', time() );
		return $gen;
	}

	/* ------------------------------------------------------------------ */
	/*  Media files                                                        */
	/* ------------------------------------------------------------------ */

	public static function media_path( int $id, bool $thumb ): string {
		return self::dir( 'files' ) . '/' . $id . ( $thumb ? '_t' : '' ) . '.bin';
	}

	public static function delete_media( int $id ): void {
		foreach ( [ false, true ] as $t ) {
			$p = self::media_path( $id, $t );
			if ( file_exists( $p ) ) wp_delete_file( $p );
		}
	}

	/**
	 * Appends a piece of an upload. The file becomes visible on the last piece.
	 *
	 * @return true|WP_Error
	 */
	public static function write_chunk( string $final_path, string $bytes, int $offset, bool $final ) {
		$part = $final_path . '.part';
		if ( $offset === 0 ) {
			if ( file_exists( $part ) ) wp_delete_file( $part );
		} elseif ( ! file_exists( $part ) || filesize( $part ) < $offset ) {
			return new WP_Error( 'myroots_upload_order', 'upload pieces out of order', [ 'status' => 409 ] );
		} elseif ( filesize( $part ) > $offset ) {
			// The same piece sent again after a lost answer: drop the first copy.
			// phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_fopen
			$h = fopen( $part, 'r+' );
			if ( ! $h || ! ftruncate( $h, $offset ) ) {
				return new WP_Error( 'myroots_write', 'cannot write to the uploads folder', [ 'status' => 500 ] );
			}
			fclose( $h ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_fclose
			clearstatcache( true, $part );
		}
		// phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_file_put_contents
		if ( file_put_contents( $part, $bytes, FILE_APPEND | LOCK_EX ) === false ) {
			return new WP_Error( 'myroots_write', 'cannot write to the uploads folder', [ 'status' => 500 ] );
		}
		clearstatcache( true, $part );
		if ( $final && ! rename( $part, $final_path ) ) { // phpcs:ignore WordPress.WP.AlternativeFunctions.rename_rename
			return new WP_Error( 'myroots_write', 'cannot write to the uploads folder', [ 'status' => 500 ] );
		}
		return true;
	}

	/** Sends a file with long caching (file contents never change for a given URL). */
	public static function send_file( string $path, string $mime, bool $private ): void {
		if ( ! file_exists( $path ) ) {
			status_header( 404 );
			exit;
		}
		$size = filesize( $path );
		$etag = '"' . md5( $path . filemtime( $path ) . $size ) . '"';
		while ( ob_get_level() ) ob_end_clean();
		header( 'Content-Type: ' . $mime );
		header( 'X-Content-Type-Options: nosniff' );
		header( 'Cache-Control: ' . ( $private ? 'private' : 'public' ) . ', max-age=31536000, immutable' );
		header( 'ETag: ' . $etag );
		header( 'Accept-Ranges: none' );
		if ( isset( $_SERVER['HTTP_IF_NONE_MATCH'] ) && trim( wp_unslash( $_SERVER['HTTP_IF_NONE_MATCH'] ) ) === $etag ) { // phpcs:ignore WordPress.Security.ValidatedSanitizedInput.InputNotSanitized
			status_header( 304 );
			exit;
		}
		header( 'Content-Length: ' . $size );
		readfile( $path ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_readfile
		exit;
	}
}
