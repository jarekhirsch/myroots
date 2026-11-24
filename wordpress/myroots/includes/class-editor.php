<?php
/**
 * MyRoots_Editor: the MyRoots app in wp-admin.
 *
 * MyRoots → Family tree opens the app full screen (its own page, without the
 * admin chrome around it, so it looks and works exactly like the desktop app).
 * The way back to WordPress is in the menu of the app.
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Editor {

	const SLUG = 'myroots';

	public static function init(): void {
		add_action( 'admin_menu', [ __CLASS__, 'menu' ] );
		add_action( 'admin_bar_menu', [ __CLASS__, 'admin_bar' ], 80 );
		add_filter( 'plugin_action_links_' . plugin_basename( MYROOTS_FILE ), [ __CLASS__, 'action_links' ] );
	}

	public static function icon(): string {
		$svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22v-7"/><path d="M12 15c-3 0-5-2-5-5a5 5 0 0 1 10 0c0 3-2 5-5 5Z"/><path d="M9 22h6"/><path d="M12 12v3"/><path d="m10 10 2 2 2-2"/></svg>';
		return 'data:image/svg+xml;base64,' . base64_encode( $svg ); // phpcs:ignore WordPress.PHP.DiscouragedPHPFunctions.obfuscation_base64_encode
	}

	public static function menu(): void {
		$cap  = current_user_can( 'manage_options' ) ? 'manage_options' : MYROOTS_CAP;
		$hook = add_menu_page( 'MyRoots', 'MyRoots', $cap, self::SLUG, '__return_null', self::icon(), 26 );
		add_submenu_page( self::SLUG, __( 'Family tree', 'myroots' ), __( 'Family tree', 'myroots' ), $cap, self::SLUG, '__return_null' );
		add_action( 'load-' . $hook, [ __CLASS__, 'render' ] );
	}

	public static function admin_bar( WP_Admin_Bar $bar ): void {
		if ( ! myroots_can_edit() ) return;
		$bar->add_node( [ 'id' => 'myroots', 'title' => '<span class="ab-icon dashicons dashicons-networking" style="top:2px"></span>MyRoots', 'href' => admin_url( 'admin.php?page=' . self::SLUG ) ] );
		$bar->add_node( [ 'parent' => 'myroots', 'id' => 'myroots-edit', 'title' => esc_html__( 'Edit the family tree', 'myroots' ), 'href' => admin_url( 'admin.php?page=' . self::SLUG ) ] );
		$bar->add_node( [ 'parent' => 'myroots', 'id' => 'myroots-site', 'title' => esc_html__( 'View the family website', 'myroots' ), 'href' => MyRoots_Site::url() ] );
	}

	public static function action_links( array $links ): array {
		array_unshift( $links, '<a href="' . esc_url( admin_url( 'admin.php?page=' . self::SLUG ) ) . '">' . esc_html__( 'Open', 'myroots' ) . '</a>' );
		return $links;
	}

	/** The interface language of the app for the WordPress user (its 12 languages). */
	private static function lang(): string {
		$l = substr( get_user_locale(), 0, 2 );
		return in_array( $l, [ 'en', 'pl', 'de', 'fr', 'es', 'it', 'pt', 'nl', 'sv', 'fi', 'da', 'he' ], true ) ? $l : 'en';
	}

	/**
	 * Files of the built app (from app/index.html of the Svelte build): the
	 * entry module and its stylesheets, as paths relative to app/.
	 */
	public static function assets(): array {
		$html = (string) file_get_contents( MYROOTS_PATH . 'app/index.html' ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_get_contents_file_get_contents -- local file of the plugin.
		$out  = [ 'js' => [], 'css' => [] ];
		if ( preg_match_all( '#<script[^>]+src="\./(assets/[A-Za-z0-9._-]+\.js)"#', $html, $m ) ) $out['js'] = $m[1];
		if ( preg_match_all( '#<link[^>]+rel="stylesheet"[^>]+href="\./(assets/[A-Za-z0-9._-]+\.css)"#', $html, $m ) ) $out['css'] = $m[1];
		return $out;
	}

	/** Prints the app page (its own document, without the admin chrome) and stops. */
	public static function render(): void {
		if ( ! myroots_can_edit() ) wp_die( esc_html__( 'Sorry, you are not allowed to access this page.', 'myroots' ), 403 );
		// the tree: from the link (e.g. in a suggestion e-mail), else the one this editor opened last
		$want = isset( $_GET['tree'] ) ? sanitize_key( wp_unslash( $_GET['tree'] ) ) : ''; // phpcs:ignore WordPress.Security.NonceVerification.Recommended -- only chooses which tree to show.
		if ( $want !== '' && MyRoots_Trees::exists( $want ) ) MyRoots_Trees::remember( $want );
		MyRoots_Trees::set( MyRoots_Trees::for_user() );
		$app = MYROOTS_URL . 'app/';
		$st  = MyRoots_Site::state();
		$cfg = [
			'rest'         => esc_url_raw( rest_url( MyRoots_Rest::NS . '/' ) ),
			'nonce'        => wp_create_nonce( 'wp_rest' ),
			'ajax'         => admin_url( 'admin-ajax.php' ),
			'assets'       => $app,
			'home'         => home_url(),
			'admin'        => admin_url(),
			'site_url'     => MyRoots_Site::url(),
			'site_name'    => get_bloginfo( 'name' ),
			'gen'          => (int) MyRoots_Trees::get_opt( 'myroots3_gen', 1 ),
			'tree'         => MyRoots_Trees::current(),
			'tree_slug'    => MyRoots_Trees::slug(),
			'legacy'       => MyRoots_Legacy::count(),
			'can_publish'  => true,
			'auto_publish' => (bool) MyRoots_Trees::get_opt( 'myroots3_auto_publish', true ),
			'published'    => (int) ( $st['published'] ?? 0 ) * 1000,
			'user'         => wp_get_current_user()->display_name,
			'lang'         => self::lang(),
			'weather_decided' => (bool) MyRoots_Trees::get_opt( 'myroots3_weather_decided', false ),
		];
		$files = self::assets();
		nocache_headers();
		header( 'Content-Type: text/html; charset=UTF-8' );
		header( 'X-Frame-Options: SAMEORIGIN' );
		?><!doctype html>
<html lang="<?php echo esc_attr( self::lang() ); ?>">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
<meta name="theme-color" content="#1d5a44" />
<meta name="robots" content="noindex" />
<title><?php echo esc_html( 'MyRoots · ' . get_bloginfo( 'name' ) ); ?></title>
<link rel="icon" type="image/svg+xml" href="<?php echo esc_url( $app . 'icon.svg' ); ?>" />
<link rel="apple-touch-icon" href="<?php echo esc_url( $app . 'apple-touch-icon.png' ); ?>" />
<link rel="manifest" href="<?php echo esc_url( MyRoots_Pwa::editor_manifest_url() ); ?>" />
<meta name="apple-mobile-web-app-capable" content="yes" />
<meta name="mobile-web-app-capable" content="yes" />
<meta name="apple-mobile-web-app-title" content="MyRoots" />
<?php
		foreach ( $files['css'] as $i => $css ) {
			wp_register_style( 'myroots-app-' . $i, $app . $css, [], MYROOTS_VERSION );
			wp_print_styles( 'myroots-app-' . $i );
		}
		wp_print_inline_script_tag( 'window.MYROOTS_WP=' . wp_json_encode( $cfg, JSON_HEX_TAG | JSON_HEX_AMP | JSON_HEX_APOS | JSON_HEX_QUOT ) . ';' );
		foreach ( $files['js'] as $js ) {
			wp_print_script_tag( [ 'type' => 'module', 'src' => esc_url( $app . $js ) ] );
		}
		?>
</head>
<body>
<div id="app"></div>
</body>
</html>
		<?php
		exit;
	}
}
