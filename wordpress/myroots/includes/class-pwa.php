<?php
/**
 * MyRoots_Pwa: the family website and the editor as installable apps.
 *
 * The family website of each tree gets a web app manifest and a service
 * worker (its scope is the address of that tree), so relatives can add it to
 * the home screen or the dock and open it offline once they have seen it.
 * The editor in wp-admin gets a manifest of its own, so it can be installed
 * as a separate window too (it needs the connection to WordPress to work).
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Pwa {

	const THEME = '#1d5a44';
	const BG    = '#f6f8f6';

	private static function icons(): array {
		$app = MYROOTS_URL . 'app/';
		return [
			[ 'src' => $app . 'icon-192.png', 'sizes' => '192x192', 'type' => 'image/png' ],
			[ 'src' => $app . 'icon-512.png', 'sizes' => '512x512', 'type' => 'image/png' ],
			[ 'src' => $app . 'icon-512.png', 'sizes' => '512x512', 'type' => 'image/png', 'purpose' => 'maskable' ],
			[ 'src' => $app . 'icon.svg', 'sizes' => 'any', 'type' => 'image/svg+xml' ],
		];
	}

	private static function lang(): string {
		return str_replace( '_', '-', get_locale() );
	}

	private static function send_json( array $m ): void {
		while ( ob_get_level() ) ob_end_clean();
		header( 'Content-Type: application/manifest+json; charset=UTF-8' );
		header( 'Cache-Control: public, max-age=3600' );
		header( 'X-Content-Type-Options: nosniff' );
		echo wp_json_encode( $m, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE );
		exit;
	}

	/** Manifest of the family website of the current tree. */
	public static function site_manifest(): void {
		$name  = MyRoots_Trees::name() !== '' ? MyRoots_Trees::name() : get_bloginfo( 'name' );
		$start = MyRoots_Site::url();
		self::send_json( [
			'id'               => $start,
			'name'             => $name !== '' ? $name : 'MyRoots',
			'short_name'       => mb_substr( $name !== '' ? $name : 'MyRoots', 0, 24 ),
			'description'      => __( 'Family tree', 'myroots' ),
			'lang'             => self::lang(),
			'start_url'        => $start,
			'scope'            => MyRoots_Site::base(),
			'display'          => 'standalone',
			'background_color' => self::BG,
			'theme_color'      => self::THEME,
			'icons'            => self::icons(),
		] );
	}

	/** Manifest of the editor (wp-admin). */
	public static function editor_manifest(): void {
		$start = admin_url( 'admin.php?page=' . MyRoots_Editor::SLUG );
		self::send_json( [
			'id'               => $start,
			'name'             => 'MyRoots · ' . get_bloginfo( 'name' ),
			'short_name'       => 'MyRoots',
			'description'      => __( 'Edit the family tree', 'myroots' ),
			'lang'             => self::lang(),
			'start_url'        => $start,
			'scope'            => admin_url( '/' ),
			'display'          => 'standalone',
			'background_color' => self::BG,
			'theme_color'      => self::THEME,
			'icons'            => self::icons(),
		] );
	}

	/** Where the editor manifest is served (the main address of the website). */
	public static function editor_manifest_url(): string {
		$keep = MyRoots_Trees::current();
		MyRoots_Trees::set( MyRoots_Trees::main() );
		$url = MyRoots_Site::base() . 'editor.webmanifest';
		MyRoots_Trees::set( $keep );
		return $url;
	}

	/**
	 * The service worker of a tree's website: pages and data from the network
	 * when there is one (always the newest version), from the cache offline;
	 * the files of the app and the photos from the cache (they never change).
	 */
	public static function service_worker(): void {
		$st    = MyRoots_Site::state();
		$cache = 'myroots-' . MyRoots_Trees::current() . '-' . MYROOTS_VERSION . '-' . (int) ( $st['published'] ?? 0 );
		$cfg   = [
			'cache'  => $cache,
			'prefix' => 'myroots-' . MyRoots_Trees::current() . '-',
			'start'  => MyRoots_Site::url(),
			'app'    => wp_make_link_relative( MYROOTS_URL . 'app/' ),
			'media'  => wp_make_link_relative( MyRoots_Site::base() . 'media/' ),
		];
		while ( ob_get_level() ) ob_end_clean();
		header( 'Content-Type: application/javascript; charset=UTF-8' );
		header( 'Cache-Control: no-cache' );
		header( 'X-Content-Type-Options: nosniff' );
		echo 'const C=' . wp_json_encode( $cfg, JSON_UNESCAPED_SLASHES ) . ";\n";
		// phpcs:ignore WordPress.WP.AlternativeFunctions.file_get_contents_file_get_contents -- local file of the plugin.
		echo file_get_contents( MYROOTS_PATH . 'assets/sw.js' ); // phpcs:ignore WordPress.Security.EscapeOutput.OutputNotEscaped -- static script of the plugin.
		exit;
	}

	/** Tags in <head> of the website page. */
	public static function head_tags(): void {
		printf( '<link rel="manifest" href="%s" />' . "\n", esc_url( MyRoots_Site::base() . 'manifest.webmanifest' ) );
		printf( '<meta name="theme-color" content="%s" />' . "\n", esc_attr( self::THEME ) );
		printf( '<meta name="apple-mobile-web-app-capable" content="yes" />' . "\n" );
		printf( '<meta name="mobile-web-app-capable" content="yes" />' . "\n" );
		printf( '<meta name="apple-mobile-web-app-title" content="%s" />' . "\n", esc_attr( MyRoots_Trees::name() !== '' ? MyRoots_Trees::name() : get_bloginfo( 'name' ) ) );
		printf( '<link rel="apple-touch-icon" href="%s" />' . "\n", esc_url( MYROOTS_URL . 'app/apple-touch-icon.png' ) );
	}

	/** Registers the service worker (only on the full-screen page, not in the embedded frame). */
	public static function register_script(): void {
		$sw = MyRoots_Site::base() . 'sw.js';
		wp_print_inline_script_tag( 'if("serviceWorker" in navigator&&(location.protocol==="https:"||location.hostname==="localhost")){addEventListener("load",function(){navigator.serviceWorker.register(' . wp_json_encode( $sw ) . ',{scope:' . wp_json_encode( MyRoots_Site::base() ) . '}).catch(function(){})})}' );
	}
}
