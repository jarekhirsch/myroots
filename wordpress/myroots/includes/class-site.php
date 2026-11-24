<?php
/**
 * MyRoots_Site: the public, read-only family website.
 *
 * It is the MyRoots app itself in read-only mode (the same files as the
 * editor, no second copy), shown in pages through the [myroots] shortcode or
 * the MyRoots block. The data is published from the editor: the engine
 * applies the privacy settings and only the public data and the photos it
 * needs are sent to WordPress.
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Site {

	const QV = 'myroots_site';

	const LEGACY = [
		'myrootstree'          => '/tree',
		'myrootspets'          => '/pets',
		'myrootsmap'           => '/map',
		'myrootslifeline'      => '/lifeline',
		'myrootsevents'        => '/events',
		'myrootsanniversaries' => '/events?tab=anniversaries',
		'myrootsstats'         => '/stats',
		'myrootsgallery'       => '/gallery',
		'myrootsbook'          => '/book',
		'myrootskinship'       => '/kinship',
		'myrootsmerge'         => '',
	];

	public static function init(): void {
		add_action( 'init', [ __CLASS__, 'rewrite' ] );
		// updates replace the plugin without activation: refresh the rules once
		add_action( 'init', static function () {
			$key = MYROOTS_VERSION . '|' . self::slug();
			if ( get_option( 'myroots3_rules' ) !== $key ) {
				flush_rewrite_rules( false );
				update_option( 'myroots3_rules', $key );
			}
		}, 99 );
		add_filter( 'query_vars', static function ( $v ) {
			$v[] = self::QV;
			return $v;
		} );
		add_action( 'parse_request', [ __CLASS__, 'serve' ], 1 );
		add_shortcode( 'myroots', [ __CLASS__, 'shortcode' ] );
		// shortcodes of MyRoots 2.x: the pages made by the old version keep working
		foreach ( self::LEGACY as $tag => $view ) {
			add_shortcode( $tag, static function ( $atts ) use ( $view ) {
				if ( $view === '' ) return '';
				$a = shortcode_atts( [ 'height' => '85vh', 'tree' => '' ], (array) $atts );
				return self::embed_html( $view, (string) $a['height'], (string) $a['tree'] );
			} );
		}
		add_action( 'init', [ __CLASS__, 'block' ] );
		add_filter( 'redirect_canonical', static function ( $redirect, $requested ) {
			return strpos( (string) $requested, self::slug() ) !== false && get_query_var( self::QV, null ) !== null ? false : $redirect;
		}, 10, 2 );
	}

	public static function slug(): string {
		$s = (string) get_option( 'myroots3_slug', '' );
		if ( $s === '' ) {
			$s = strpos( get_locale(), 'pl' ) === 0 ? 'drzewo' : 'family-tree';
			// a page with that address (e.g. created by MyRoots 2.x) keeps it
			if ( did_action( 'init' ) && get_page_by_path( $s ) ) $s .= '-myroots';
			if ( did_action( 'init' ) ) update_option( 'myroots3_slug', $s );
		}
		return trim( $s, '/' );
	}

	public static function rewrite(): void {
		$slug = preg_quote( self::slug(), '#' );
		add_rewrite_rule( '^' . $slug . '/?$', 'index.php?' . self::QV . '=index', 'top' );
		add_rewrite_rule( '^' . $slug . '/(.+)$', 'index.php?' . self::QV . '=$matches[1]', 'top' );
	}

	/**
	 * Base address of the website of the current tree (ends with "/" or "=",
	 * ready for file names). The main tree is at /drzewo/, others at /drzewo/<slug>/.
	 */
	public static function base(): string {
		$tree = MyRoots_Trees::slug();
		$tree = $tree === '' ? '' : $tree . '/';
		if ( get_option( 'permalink_structure' ) ) return home_url( '/' . self::slug() . '/' . $tree );
		return home_url( '/?' . self::QV . '=' . $tree );
	}

	public static function url(): string {
		return get_option( 'permalink_structure' ) ? self::base() : self::base() . 'index';
	}

	public static function state(): array {
		return (array) MyRoots_Trees::get_opt( 'myroots3_site', [] );
	}

	/** Chooses the tree from the first part of the address; returns the rest. */
	private static function tree_from_path( string $path ): string {
		$parts = explode( '/', $path, 2 );
		$id    = $parts[0] !== '' ? MyRoots_Trees::by_slug( $parts[0] ) : null;
		if ( $id === null ) {
			// the address a tree had before it became the main one keeps working
			foreach ( MyRoots_Trees::all() as $tid => $t ) {
				if ( $parts[0] !== '' && (string) ( $t['slug'] ?? '' ) === $parts[0] ) $id = (string) $tid;
			}
		}
		if ( $id === null ) {
			MyRoots_Trees::set( MyRoots_Trees::main() );
			return $path;
		}
		MyRoots_Trees::set( $id );
		return $parts[1] ?? '';
	}

	/** Who may see the website: everybody, logged-in users, or nobody. */
	public static function visible_for_current_user(): bool {
		$v = (string) get_option( 'myroots3_visibility', 'public' );
		if ( myroots_can_edit() ) return true;
		if ( $v === 'users' ) return is_user_logged_in();
		return $v === 'public';
	}

	/* ------------------------------------------------------------------ */
	/*  Publishing                                                         */
	/* ------------------------------------------------------------------ */

	/** Switches to the uploaded version and removes photos no longer public. */
	public static function finish( array $sigs ) {
		$dir  = MyRoots_Store::dir( 'site' );
		$next = $dir . '/data.next.js';
		if ( ! file_exists( $next ) ) return new WP_Error( 'myroots_publish', 'data missing', [ 'status' => 400 ] );
		// people on the website (shown in the editor)
		$persons = 0;
		$js      = (string) file_get_contents( $next ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_get_contents_file_get_contents
		$persons = substr_count( $js, '{"kind":"person"' );
		// the whole website protected with a password: the number is in the header
		if ( preg_match( '#"lock":\{"v":1,"mode":"all".*?"people":(\d+)#s', $js, $lm ) ) $persons = (int) $lm[1];
		if ( ! rename( $next, $dir . '/data.js' ) ) return new WP_Error( 'myroots_publish', 'cannot write', [ 'status' => 500 ] ); // phpcs:ignore WordPress.WP.AlternativeFunctions.rename_rename
		foreach ( (array) glob( $dir . '/media/*' ) as $f ) {
			if ( ! is_string( $f ) ) continue;
			$id = (int) basename( $f );
			if ( ! isset( $sigs[ $id ] ) || substr( $f, -5 ) === '.part' ) wp_delete_file( $f );
		}
		MyRoots_Trees::update_opt( 'myroots3_site', [ 'published' => time(), 'sigs' => $sigs, 'persons' => $persons ] );
		return true;
	}

	/* ------------------------------------------------------------------ */
	/*  Serving                                                            */
	/* ------------------------------------------------------------------ */

	public static function serve( WP $wp ): void {
		if ( ! array_key_exists( self::QV, $wp->query_vars ) ) return;
		$path = (string) $wp->query_vars[ self::QV ];
		$path = self::tree_from_path( ltrim( $path, '/' ) );
		if ( $path === 'index' ) $path = '';
		// installable app: names and icons only, so no login is needed for them
		if ( $path === 'editor.webmanifest' ) MyRoots_Pwa::editor_manifest();
		if ( $path === 'manifest.webmanifest' ) MyRoots_Pwa::site_manifest();
		if ( $path === 'sw.js' ) MyRoots_Pwa::service_worker();
		if ( (string) get_option( 'myroots3_visibility', 'public' ) === 'off' && ! myroots_can_edit() ) {
			status_header( 404 );
			nocache_headers();
			exit;
		}
		if ( ! self::visible_for_current_user() ) {
			if ( $path === '' || $path === 'embed' ) {
				wp_safe_redirect( wp_login_url( self::url() ) );
			} else {
				status_header( 403 );
			}
			exit;
		}
		$dir     = MyRoots_Store::dir( 'site' );
		$private = (string) get_option( 'myroots3_visibility', 'public' ) !== 'public';
		if ( $path === 'data.js' ) {
			header( 'Content-Type: application/javascript; charset=UTF-8' );
			header( 'Cache-Control: ' . ( $private ? 'private' : 'public' ) . ', max-age=60' );
			header( 'X-Content-Type-Options: nosniff' );
			$f = $dir . '/data.js';
			if ( file_exists( $f ) ) self::send_text_gz( $f );
			else echo 'window.MYROOTS_SITE=null;';
			exit;
		}
		if ( preg_match( '#^media/(\d+(?:_t)?)\.([a-z0-9]{2,5})$#', $path, $m ) ) {
			$file = $dir . '/media/' . $m[1] . '.' . $m[2];
			$mime = wp_check_filetype( 'x.' . $m[2] );
			$type = $mime['type'] ?: 'application/octet-stream';
			if ( strpos( $type, 'svg' ) !== false ) $type = 'application/octet-stream';
			header( "Content-Security-Policy: default-src 'none'; sandbox" );
			MyRoots_Store::send_file( $file, $type, $private );
		}
		if ( $path === '' || $path === 'embed' ) self::page( $path === 'embed' );
		status_header( 404 );
		exit;
	}

	/** The website page: the viewer of the app, standalone (no theme around it). */
	private static function page( bool $embed ): void {
		$st    = self::state();
		$ver   = MYROOTS_VERSION . '.' . (int) ( $st['published'] ?? 0 );
		// the app itself (the same files as the editor), started read-only
		$view  = MYROOTS_URL . 'app/';
		$files = MyRoots_Editor::assets();
		$title = MyRoots_Trees::name() !== '' ? MyRoots_Trees::name() : get_bloginfo( 'name' );
		$cfg   = [
			'media'     => self::base() . 'media/',
			'home'      => $embed ? '' : home_url( '/' ),
			'site_name' => $title,
			'suggest'   => add_query_arg( [ '_wpnonce' => is_user_logged_in() ? wp_create_nonce( 'wp_rest' ) : null, 'tree' => MyRoots_Trees::current() ], rest_url( MyRoots_Rest::NS . '/suggest' ) ),
		];
		nocache_headers();
		header( 'Content-Type: text/html; charset=UTF-8' );
		header( 'X-Content-Type-Options: nosniff' );
		if ( ! $embed ) header( 'X-Frame-Options: SAMEORIGIN' );
		$index = get_option( 'myroots3_index', false );
		?><!doctype html>
<html lang="<?php echo esc_attr( substr( get_locale(), 0, 2 ) ); ?>">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover" />
<?php if ( ! $index || $embed ) : ?><meta name="robots" content="noindex" />
<?php endif; ?>
<title><?php echo esc_html( $title ); ?></title>
<link rel="icon" type="image/svg+xml" href="<?php echo esc_url( $view . 'icon.svg' ); ?>" />
<?php
		if ( ! $embed ) MyRoots_Pwa::head_tags();
		foreach ( $files['css'] as $i => $css ) {
			wp_register_style( 'myroots-view-' . $i, $view . $css, [], MYROOTS_VERSION );
			wp_print_styles( 'myroots-view-' . $i );
		}
		?>
<?php if ( $embed ) : ?><style>html,body{background:transparent}</style>
<?php endif; ?>
<?php
		$flags = JSON_HEX_TAG | JSON_HEX_AMP | JSON_HEX_APOS | JSON_HEX_QUOT;
		wp_print_inline_script_tag( 'window.MYROOTS_ASSETS=' . wp_json_encode( $view, $flags ) . ';window.MYROOTS_WP_VIEW=' . wp_json_encode( $cfg, $flags ) . ';' );
		?>
<?php if ( empty( $st['published'] ) ) : ?>
</head>
<body style="font-family:system-ui,sans-serif;display:flex;min-height:100vh;align-items:center;justify-content:center;text-align:center;padding:24px;color:#2b2b2b;background:#faf8f3">
<div><img src="<?php echo esc_url( $view . 'icon.svg' ); ?>" alt="" width="56" height="56" />
<h1 style="font-size:1.3rem"><?php esc_html_e( 'The family tree is not published yet.', 'myroots' ); ?></h1>
<?php if ( myroots_can_edit() ) : ?><p><a href="<?php echo esc_url( admin_url( 'admin.php?page=myroots&tree=' . MyRoots_Trees::current() . '#/website' ) ); ?>"><?php esc_html_e( 'Open MyRoots and publish it', 'myroots' ); ?></a></p><?php endif; ?>
</div></body></html>
			<?php
			exit;
		endif;
		?>
<?php
		wp_print_script_tag( [ 'src' => esc_url( self::base() . 'data.js?v=' . $ver ) ] );
		if ( ! $embed ) MyRoots_Pwa::register_script();
		foreach ( $files['js'] as $js ) {
			wp_print_script_tag( [ 'type' => 'module', 'src' => esc_url( $view . $js ) ] );
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

	/* ------------------------------------------------------------------ */
	/*  Inside pages: shortcode and block                                  */
	/* ------------------------------------------------------------------ */

	/**
	 * [myroots view="/tree" height="85vh" tree="kowalscy"]
	 * view: start view of the app (/, /people, /tree, /map, /stats, /gallery, /person/12 …).
	 * tree: address of another family tree (see MyRoots → Settings); empty = the main tree.
	 */
	public static function shortcode( $atts ): string {
		$a = shortcode_atts( [ 'view' => '/', 'height' => '85vh', 'tree' => '' ], (array) $atts, 'myroots' );
		return self::embed_html( (string) $a['view'], (string) $a['height'], (string) $a['tree'] );
	}

	public static function embed_html( string $view, string $height, string $tree = '' ): string {
		$keep = MyRoots_Trees::current();
		$tree = sanitize_title( $tree );
		$id   = $tree === '' ? MyRoots_Trees::main() : ( MyRoots_Trees::by_slug( $tree ) ?? ( MyRoots_Trees::exists( $tree ) ? $tree : MyRoots_Trees::main() ) );
		MyRoots_Trees::set( $id );
		$html = self::embed_markup( $view, $height );
		MyRoots_Trees::set( $keep );
		return $html;
	}

	private static function embed_markup( string $view, string $height ): string {
		$view   = '/' . ltrim( preg_replace( '#[^A-Za-z0-9/_?=&.-]#', '', $view ), '/' );
		$height = preg_match( '#^\d+(\.\d+)?(px|vh|em|rem|%)$#', $height ) ? $height : '85vh';
		$src    = self::base() . 'embed#' . $view;
		return sprintf(
			'<div class="myroots-embed" style="width:100%%;height:%1$s;border-radius:14px;overflow:hidden;border:1px solid rgba(0,0,0,.08)"><iframe src="%2$s" title="%3$s" style="width:100%%;height:100%%;border:0;display:block" loading="lazy" allow="fullscreen; clipboard-write"></iframe></div><p class="myroots-embed-link" style="font-size:.85em;margin:.4em 0 0"><a href="%4$s" target="_blank" rel="noopener">%5$s</a></p>',
			esc_attr( $height ),
			esc_url( $src ),
			esc_attr__( 'Family tree', 'myroots' ),
			esc_url( self::url() . '#' . $view ),
			esc_html__( 'Open the family tree in full screen', 'myroots' )
		);
	}

	public static function block(): void {
		if ( ! function_exists( 'register_block_type' ) ) return;
		wp_register_script( 'myroots-block', MYROOTS_URL . 'blocks/tree/index.js', [ 'wp-blocks', 'wp-element', 'wp-block-editor', 'wp-components', 'wp-i18n', 'wp-server-side-render' ], MYROOTS_VERSION, true );
		wp_set_script_translations( 'myroots-block', 'myroots', MYROOTS_PATH . 'languages' );
		if ( is_admin() ) {
			$list = [];
			foreach ( MyRoots_Trees::export() as $i => $t ) {
				/* translators: %d: number of the family tree */
				$list[] = [ 'slug' => $t['slug'], 'name' => $t['name'] !== '' ? $t['name'] : sprintf( __( 'Family tree %d', 'myroots' ), $i + 1 ) ];
			}
			wp_add_inline_script( 'myroots-block', 'window.MYROOTS_TREES=' . wp_json_encode( $list ) . ';', 'before' );
		}
		register_block_type( MYROOTS_PATH . 'blocks/tree', [
			'editor_script'   => 'myroots-block',
			'render_callback' => static function ( $attrs ) {
				return self::embed_html( (string) ( $attrs['view'] ?? '/' ), (string) ( $attrs['height'] ?? '85vh' ), (string) ( $attrs['tree'] ?? '' ) );
			},
		] );
	}

	/**
	 * Sends a big text file (data.js, tens of MB for big trees) compressed when
	 * the browser accepts it: a .gz copy is made once, in pieces, next to it.
	 */
	private static function send_text_gz( string $path ): void {
		header( 'Vary: Accept-Encoding' );
		$accept = isset( $_SERVER['HTTP_ACCEPT_ENCODING'] ) ? sanitize_text_field( wp_unslash( $_SERVER['HTTP_ACCEPT_ENCODING'] ) ) : '';
		$gzip   = stripos( $accept, 'gzip' ) !== false && function_exists( 'gzopen' ) && ! ini_get( 'zlib.output_compression' );
		if ( $gzip ) {
			$gz = $path . '.gz';
			if ( ! file_exists( $gz ) || filemtime( $gz ) <= filemtime( $path ) ) {
				$tmp = $gz . '.' . wp_generate_password( 6, false );
				// phpcs:disable WordPress.WP.AlternativeFunctions -- streamed copy of a big file, not loaded into memory.
				$in  = fopen( $path, 'rb' );
				$out = $in ? gzopen( $tmp, 'wb6' ) : false;
				if ( $in && $out ) {
					while ( ! feof( $in ) ) gzwrite( $out, (string) fread( $in, 1 << 20 ) );
					gzclose( $out );
					fclose( $in );
					rename( $tmp, $gz );
				}
				// phpcs:enable
				if ( file_exists( $tmp ) ) wp_delete_file( $tmp );
			}
			if ( file_exists( $gz ) ) {
				header( 'Content-Encoding: gzip' );
				header( 'Content-Length: ' . filesize( $gz ) );
				readfile( $gz ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_readfile
				return;
			}
		}
		header( 'Content-Length: ' . filesize( $path ) );
		readfile( $path ); // phpcs:ignore WordPress.WP.AlternativeFunctions.file_system_operations_readfile
	}
}
