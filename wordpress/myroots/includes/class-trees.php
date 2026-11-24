<?php
/**
 * MyRoots_Trees: several separate family trees on one WordPress site.
 *
 * Each tree has its own records table, its own folders for photos and for the
 * published website, its own options and its own public address. The first
 * tree ("default") uses the names of MyRoots 3.0, so an existing tree keeps
 * working unchanged; other trees add their id to those names.
 *
 * Every request works on one tree: the editor chooses it (REST parameter
 * "tree"), the public website takes it from its address (/drzewo/<slug>/).
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Trees {

	const OPT     = 'myroots3_trees';
	const DEF     = 'default';
	const USERKEY = 'myroots3_tree';
	/** Path names used by the website itself: never a tree address. */
	const RESERVED = [ 'index', 'embed', 'media', 'data', 'manifest', 'sw', 'icon', 'assets', 'app' ];

	private static $current = null;

	/** The list: id => [name, slug, created]. Never empty. */
	public static function all(): array {
		$t = get_option( self::OPT );
		if ( ! is_array( $t ) || ! $t ) {
			$t = [ self::DEF => [ 'name' => '', 'slug' => '', 'created' => 0 ] ];
		}
		return $t;
	}

	public static function exists( string $id ): bool {
		return isset( self::all()[ $id ] );
	}

	/** The tree shown at the main address of the website (the first one in the list). */
	public static function main(): string {
		$t = self::all();
		return isset( $t[ self::DEF ] ) ? self::DEF : (string) array_key_first( $t );
	}

	public static function current(): string {
		if ( self::$current === null || ! self::exists( self::$current ) ) self::$current = self::main();
		return self::$current;
	}

	public static function set( string $id ): bool {
		if ( ! self::exists( $id ) ) return false;
		self::$current = $id;
		return true;
	}

	/** Suffix of table, folder and option names (empty for the first tree). */
	public static function suffix( ?string $id = null ): string {
		$id = $id ?? self::current();
		return $id === self::DEF ? '' : '_' . $id;
	}

	public static function opt_name( string $base, ?string $id = null ): string {
		return $base . self::suffix( $id );
	}

	public static function get_opt( string $base, $default = false ) {
		return get_option( self::opt_name( $base ), $default );
	}

	public static function update_opt( string $base, $value ): void {
		update_option( self::opt_name( $base ), $value, false );
	}

	public static function name( ?string $id = null ): string {
		$id = $id ?? self::current();
		return (string) ( self::all()[ $id ]['name'] ?? '' );
	}

	/** Address segment of a tree; empty for the main tree (main address). */
	public static function slug( ?string $id = null ): string {
		$id = $id ?? self::current();
		if ( $id === self::main() ) return '';
		$s = (string) ( self::all()[ $id ]['slug'] ?? '' );
		return $s !== '' ? $s : $id;
	}

	public static function by_slug( string $slug ): ?string {
		foreach ( self::all() as $id => $t ) {
			if ( $id !== self::main() && self::slug( (string) $id ) === $slug ) return (string) $id;
		}
		return null;
	}

	/** The tree last opened by this editor. */
	public static function for_user(): string {
		$id = (string) get_user_meta( get_current_user_id(), self::USERKEY, true );
		return self::exists( $id ) ? $id : self::main();
	}

	public static function remember( string $id ): void {
		if ( self::exists( $id ) ) update_user_meta( get_current_user_id(), self::USERKEY, $id );
	}

	private static function unique_slug( string $name, array $trees ): string {
		$base = sanitize_title( $name );
		if ( $base === '' || in_array( $base, self::RESERVED, true ) ) $base = 'tree-' . $base;
		$base = trim( substr( $base, 0, 60 ), '-' );
		$used = [];
		foreach ( $trees as $t ) $used[] = (string) ( $t['slug'] ?? '' );
		$s = $base;
		for ( $n = 2; in_array( $s, $used, true ); $n++ ) $s = $base . '-' . $n;
		return $s;
	}

	/** A new empty tree. Returns its id. */
	public static function create( string $name ): string {
		$trees = self::all();
		do {
			$id = 't' . strtolower( base_convert( (string) time(), 10, 36 ) . wp_generate_password( 4, false, false ) );
			$id = preg_replace( '/[^a-z0-9]/', '', strtolower( $id ) );
		} while ( isset( $trees[ $id ] ) );
		$name         = sanitize_text_field( $name );
		$trees[ $id ] = [ 'name' => $name, 'slug' => self::unique_slug( $name, $trees ), 'created' => time() ];
		update_option( self::OPT, $trees, false );
		MyRoots_Store::install_tree( $id );
		self::flush_rules();
		return $id;
	}

	/** New name (the address stays, links given to the family keep working). */
	public static function rename( string $id, string $name ): bool {
		$trees = self::all();
		if ( ! isset( $trees[ $id ] ) ) return false;
		$trees[ $id ]['name'] = sanitize_text_field( $name );
		update_option( self::OPT, $trees, false );
		return true;
	}

	/** Deletes a tree with its records, photos, website and options. */
	public static function delete( string $id ): bool {
		$trees = self::all();
		if ( ! isset( $trees[ $id ] ) || count( $trees ) < 2 ) return false;
		MyRoots_Store::drop_tree( $id );
		foreach ( [ 'myroots3_gen', 'myroots3_changed', 'myroots3_site', 'myroots3_auto_publish', 'myroots3_inbox' ] as $o ) {
			delete_option( self::opt_name( $o, $id ) );
		}
		unset( $trees[ $id ] );
		update_option( self::OPT, $trees, false );
		self::flush_rules();
		return true;
	}

	private static function flush_rules(): void {
		delete_option( 'myroots3_rules' );
	}

	/** For the editor and the settings page. */
	public static function export(): array {
		$out  = [];
		$keep = self::current();
		foreach ( self::all() as $id => $t ) {
			$id = (string) $id;
			self::$current = $id;
			$out[]         = [
				'id'      => $id,
				'name'    => (string) ( $t['name'] ?? '' ),
				'created' => (int) ( $t['created'] ?? 0 ) * 1000,
				'slug'    => self::slug( $id ),
				'url'     => MyRoots_Site::url(),
			];
		}
		self::$current = $keep;
		return $out;
	}
}
