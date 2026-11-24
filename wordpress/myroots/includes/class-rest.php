<?php
/**
 * MyRoots_Rest: the REST API used by the editor (wp-admin) and by the
 * suggestion form of the public family website.
 *
 * Namespace myroots/v3. Everything except POST /suggest requires a logged-in
 * user allowed to edit the tree (cookie + REST nonce).
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Rest {

	const NS = 'myroots/v3';

	public static function init(): void {
		add_action( 'rest_api_init', [ __CLASS__, 'routes' ] );
		add_action( 'wp_ajax_myroots_nonce', [ __CLASS__, 'ajax_nonce' ] );
		add_filter( 'rest_request_before_callbacks', [ __CLASS__, 'pick_tree' ], 10, 3 );
	}

	/** Every request of the editor works on one tree: the "tree" parameter. */
	public static function pick_tree( $response, $handler, WP_REST_Request $request ) {
		if ( strpos( $request->get_route(), '/' . self::NS . '/' ) !== 0 ) return $response;
		$tree = (string) $request->get_param( 'tree' );
		if ( $tree === '' ) {
			MyRoots_Trees::set( MyRoots_Trees::main() );
		} elseif ( ! MyRoots_Trees::set( $tree ) ) {
			return new WP_Error( 'myroots_tree', 'no such family tree', [ 'status' => 404 ] );
		}
		return $response;
	}

	public static function can(): bool {
		return myroots_can_edit();
	}

	public static function routes(): void {
		$edit = [ __CLASS__, 'can' ];
		$id   = '(?P<id>\d+)';

		register_rest_route( self::NS, '/records', [
			[ 'methods' => 'GET', 'callback' => [ __CLASS__, 'records_get' ], 'permission_callback' => $edit ],
			[ 'methods' => 'POST', 'callback' => [ __CLASS__, 'records_post' ], 'permission_callback' => $edit ],
		] );
		register_rest_route( self::NS, '/reset', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'reset' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/media/' . $id, [
			[ 'methods' => 'GET', 'callback' => [ __CLASS__, 'media_get' ], 'permission_callback' => $edit ],
			[ 'methods' => 'POST', 'callback' => [ __CLASS__, 'media_post' ], 'permission_callback' => $edit ],
		] );
		$tid = '(?P<tid>[a-z0-9]+)';
		register_rest_route( self::NS, '/trees', [
			[ 'methods' => 'GET', 'callback' => [ __CLASS__, 'trees_get' ], 'permission_callback' => $edit ],
			[ 'methods' => 'POST', 'callback' => [ __CLASS__, 'trees_create' ], 'permission_callback' => $edit ],
		] );
		register_rest_route( self::NS, '/trees/' . $tid, [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'trees_rename' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/trees/' . $tid . '/delete', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'trees_delete' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/trees/' . $tid . '/open', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'trees_open' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/legacy', [ 'methods' => 'GET', 'callback' => [ __CLASS__, 'legacy' ], 'permission_callback' => $edit ] );

		register_rest_route( self::NS, '/site', [ 'methods' => 'GET', 'callback' => [ __CLASS__, 'site_state' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/site/options', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'site_options' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/site/files', [ 'methods' => 'GET', 'callback' => [ __CLASS__, 'site_files' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/site/begin', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'site_begin' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/site/file', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'site_file' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/site/finish', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'site_finish' ], 'permission_callback' => $edit ] );

		register_rest_route( self::NS, '/suggestions', [ 'methods' => 'GET', 'callback' => [ __CLASS__, 'suggestions_get' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/suggestions/done', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'suggestions_done' ], 'permission_callback' => $edit ] );
		register_rest_route( self::NS, '/suggest', [ 'methods' => 'POST', 'callback' => [ __CLASS__, 'suggest' ], 'permission_callback' => '__return_true' ] );
	}

	/** A fresh REST nonce for a long editing session (cookie-authenticated). */
	public static function ajax_nonce(): void {
		if ( ! myroots_can_edit() ) wp_send_json( [ 'nonce' => '' ], 403 );
		wp_send_json( [ 'nonce' => wp_create_nonce( 'wp_rest' ) ] );
	}

	private static function body_json( WP_REST_Request $r ): array {
		$v = json_decode( $r->get_body(), true );
		return is_array( $v ) ? $v : [];
	}

	/* ------------------------------------------------------------------ */
	/*  Several trees                                                      */
	/* ------------------------------------------------------------------ */

	public static function trees_get(): array {
		return [ 'current' => MyRoots_Trees::for_user(), 'trees' => MyRoots_Trees::export() ];
	}

	public static function trees_create( WP_REST_Request $r ) {
		$b = self::body_json( $r );
		if ( count( MyRoots_Trees::all() ) >= 50 ) return new WP_Error( 'myroots_tree', 'too many family trees', [ 'status' => 400 ] );
		return MyRoots_Trees::create( (string) ( $b['name'] ?? '' ) );
	}

	public static function trees_rename( WP_REST_Request $r ) {
		$b = self::body_json( $r );
		if ( ! MyRoots_Trees::rename( (string) $r['tid'], (string) ( $b['name'] ?? '' ) ) ) return new WP_Error( 'myroots_tree', 'no such family tree', [ 'status' => 404 ] );
		return [ 'ok' => true ];
	}

	public static function trees_delete( WP_REST_Request $r ) {
		$id = (string) $r['tid'];
		if ( $id === MyRoots_Trees::for_user() ) return new WP_Error( 'myroots_tree', 'open another tree first', [ 'status' => 400 ] );
		if ( ! MyRoots_Trees::delete( $id ) ) return new WP_Error( 'myroots_tree', 'no such family tree', [ 'status' => 404 ] );
		return [ 'ok' => true ];
	}

	public static function trees_open( WP_REST_Request $r ) {
		$id = (string) $r['tid'];
		if ( ! MyRoots_Trees::exists( $id ) ) return new WP_Error( 'myroots_tree', 'no such family tree', [ 'status' => 404 ] );
		MyRoots_Trees::remember( $id );
		return [ 'ok' => true ];
	}

	/* ------------------------------------------------------------------ */

	public static function records_get(): void {
		// streamed straight from the table: big trees would not fit in PHP memory as objects
		MyRoots_Store::stream_all();
	}

	public static function records_post( WP_REST_Request $r ) {
		$b = self::body_json( $r );
		$n = MyRoots_Store::apply( (array) ( $b['records'] ?? [] ), array_map( 'intval', (array) ( $b['deleted_media'] ?? [] ) ) );
		if ( is_wp_error( $n ) ) return $n;
		return [ 'saved' => $n ];
	}

	public static function reset() {
		return [ 'gen' => MyRoots_Store::reset() ];
	}

	public static function media_post( WP_REST_Request $r ) {
		$id    = (int) $r['id'];
		$thumb = (int) $r->get_param( 'thumb' ) === 1;
		$path  = MyRoots_Store::media_path( $id, $thumb );
		$res   = MyRoots_Store::write_chunk( $path, $r->get_body(), (int) $r->get_param( 'offset' ), (int) $r->get_param( 'final' ) === 1 );
		return is_wp_error( $res ) ? $res : [ 'ok' => true ];
	}

	public static function media_get( WP_REST_Request $r ): void {
		$id     = (int) $r['id'];
		$thumb  = (int) $r->get_param( 't' ) === 1;
		$strict = (int) $r->get_param( 'strict' ) === 1;
		$path   = MyRoots_Store::media_path( $id, $thumb );
		if ( $thumb && ! file_exists( $path ) && ! $strict ) {
			$path  = MyRoots_Store::media_path( $id, false );
			$thumb = false;
		}
		$mime = 'image/jpeg';
		if ( ! $thumb ) {
			$rec  = MyRoots_Store::get( 'media', $id );
			$mime = (string) ( $rec['data']['mime'] ?? 'application/octet-stream' );
			if ( ! preg_match( '#^[a-z]+/[a-z0-9.+-]+$#i', $mime ) || stripos( $mime, 'html' ) !== false || stripos( $mime, 'svg' ) !== false || stripos( $mime, 'xml' ) !== false ) {
				$mime = 'application/octet-stream';
			}
		}
		header( "Content-Security-Policy: default-src 'none'; sandbox" );
		MyRoots_Store::send_file( $path, $mime, true );
	}

	public static function legacy(): WP_REST_Response {
		return new WP_REST_Response( MyRoots_Legacy::export() );
	}

	/* ------------------------------------------------------------------ */
	/*  Family website                                                     */
	/* ------------------------------------------------------------------ */

	public static function site_state(): array {
		$st = MyRoots_Site::state();
		return [
			'url'       => MyRoots_Site::url(),
			'published' => (int) ( $st['published'] ?? 0 ) * 1000,
			'auto'      => (bool) MyRoots_Trees::get_opt( 'myroots3_auto_publish', true ),
			'persons'   => (int) ( $st['persons'] ?? 0 ),
			'media'     => count( (array) ( $st['sigs'] ?? [] ) ),
		];
	}

	public static function site_options( WP_REST_Request $r ): array {
		$b = self::body_json( $r );
		if ( isset( $b['auto'] ) ) MyRoots_Trees::update_opt( 'myroots3_auto_publish', (bool) $b['auto'] );
		if ( ! empty( $b['weather_decided'] ) ) MyRoots_Trees::update_opt( 'myroots3_weather_decided', 1 );
		return self::site_state();
	}

	public static function site_files(): array {
		$st    = MyRoots_Site::state();
		$sigs  = (array) ( $st['sigs'] ?? [] );
		$media = MyRoots_Store::dir( 'site' ) . '/media/';
		// only photos that are really on the disk: a missing file is sent again
		foreach ( $sigs as $id => $sig ) {
			if ( ! glob( $media . (int) $id . '.*' ) && ! glob( $media . (int) $id . '_t.*' ) ) unset( $sigs[ $id ] );
		}
		return [ 'sigs' => (object) $sigs ];
	}

	public static function site_begin(): array {
		$token = wp_generate_password( 16, false, false );
		set_transient( MyRoots_Trees::opt_name( 'myroots3_publish' ), $token, HOUR_IN_SECONDS );
		return [ 'token' => $token ];
	}

	private static function check_token( string $token ): bool {
		$t = get_transient( MyRoots_Trees::opt_name( 'myroots3_publish' ) );
		return $t && hash_equals( (string) $t, $token );
	}

	public static function site_file( WP_REST_Request $r ) {
		if ( ! self::check_token( (string) $r->get_param( 'token' ) ) ) return new WP_Error( 'myroots_token', 'publishing session expired', [ 'status' => 409 ] );
		$path = (string) $r->get_param( 'path' );
		if ( $path === 'data.js' ) {
			$dest = MyRoots_Store::dir( 'site' ) . '/data.next.js';
		} elseif ( preg_match( '#^media/((\d+(?:_t)?)\.[a-z0-9]{2,5})$#', $path, $m ) ) {
			$dir = MyRoots_Store::dir( 'site' ) . '/media';
			if ( ! is_dir( $dir ) ) wp_mkdir_p( $dir );
			$dest = $dir . '/' . $m[1];
			// the same photo stored differently before (e.g. not yet encrypted): removed
			if ( (int) $r->get_param( 'offset' ) === 0 ) {
				foreach ( (array) glob( $dir . '/' . $m[2] . '.*' ) as $old ) {
					if ( is_string( $old ) && $old !== $dest && substr( $old, -5 ) !== '.part' ) wp_delete_file( $old );
				}
			}
		} else {
			return new WP_Error( 'myroots_path', 'invalid file name', [ 'status' => 400 ] );
		}
		$res = MyRoots_Store::write_chunk( $dest, $r->get_body(), (int) $r->get_param( 'offset' ), (int) $r->get_param( 'final' ) === 1 );
		return is_wp_error( $res ) ? $res : [ 'ok' => true ];
	}

	public static function site_finish( WP_REST_Request $r ) {
		$b = self::body_json( $r );
		if ( ! self::check_token( (string) ( $b['token'] ?? '' ) ) ) return new WP_Error( 'myroots_token', 'publishing session expired', [ 'status' => 409 ] );
		$sigs = [];
		foreach ( (array) ( $b['sigs'] ?? [] ) as $id => $sig ) $sigs[ (int) $id ] = substr( sanitize_text_field( (string) $sig ), 0, 80 );
		$res = MyRoots_Site::finish( $sigs );
		delete_transient( MyRoots_Trees::opt_name( 'myroots3_publish' ) );
		return is_wp_error( $res ) ? $res : self::site_state();
	}

	/* ------------------------------------------------------------------ */
	/*  Suggestions from the family                                        */
	/* ------------------------------------------------------------------ */

	public static function suggest( WP_REST_Request $r ) {
		$b    = self::body_json( $r );
		$code = isset( $b['code'] ) ? (string) $b['code'] : '';
		if ( strlen( $code ) > 20000 || ! preg_match( '#^MYROOTS-SUGGESTION:[A-Za-z0-9+/=]+:END$#', $code ) ) {
			return new WP_Error( 'myroots_suggest', 'invalid suggestion', [ 'status' => 400 ] );
		}
		if ( ! MyRoots_Site::visible_for_current_user() ) return new WP_Error( 'myroots_suggest', 'not available', [ 'status' => 403 ] );
		// a few per hour from one address, and a cap on the waiting ones
		$ip  = isset( $_SERVER['REMOTE_ADDR'] ) ? sanitize_text_field( wp_unslash( $_SERVER['REMOTE_ADDR'] ) ) : '';
		$key = 'myroots3_sg_' . md5( $ip );
		$n   = (int) get_transient( $key );
		if ( $n >= 10 ) return new WP_Error( 'myroots_suggest', 'too many suggestions, try later', [ 'status' => 429 ] );
		set_transient( $key, $n + 1, HOUR_IN_SECONDS );
		$box = (array) MyRoots_Trees::get_opt( 'myroots3_inbox', [] );
		if ( count( $box ) >= 300 ) return new WP_Error( 'myroots_suggest', 'the inbox is full', [ 'status' => 429 ] );
		$id         = wp_generate_password( 10, false, false );
		$box[ $id ] = $code;
		MyRoots_Trees::update_opt( 'myroots3_inbox', $box );

		if ( get_option( 'myroots3_notify_suggestions', true ) ) {
			$who  = sanitize_text_field( (string) ( $b['author'] ?? '' ) );
			$name = sanitize_text_field( (string) ( $b['person_name'] ?? '' ) );
			$msg  = sanitize_textarea_field( (string) ( $b['message'] ?? '' ) );
			/* translators: %s: person name */
			$subject = sprintf( __( '[MyRoots] New correction suggestion: %s', 'myroots' ), $name );
			$body    = $msg . "\n\n" . ( $who ? __( 'From:', 'myroots' ) . ' ' . $who . "\n" : '' ) . "\n" . admin_url( 'admin.php?page=myroots&tree=' . MyRoots_Trees::current() . '#/suggestions' ) . "\n";
			wp_mail( get_option( 'admin_email' ), $subject, $body );
		}
		return [ 'ok' => true ];
	}

	public static function suggestions_get(): array {
		$box = (array) MyRoots_Trees::get_opt( 'myroots3_inbox', [] );
		return [ 'codes' => array_values( $box ), 'ids' => array_keys( $box ) ];
	}

	public static function suggestions_done( WP_REST_Request $r ): array {
		$b   = self::body_json( $r );
		$box = (array) MyRoots_Trees::get_opt( 'myroots3_inbox', [] );
		foreach ( (array) ( $b['ids'] ?? [] ) as $id ) unset( $box[ (string) $id ] );
		MyRoots_Trees::update_opt( 'myroots3_inbox', $box );
		return [ 'ok' => true ];
	}
}
