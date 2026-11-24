<?php
/**
 * MyRoots_Settings: the few things that belong to WordPress, not to the tree
 * (everything about the tree itself is set in the app: MyRoots → Family tree).
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Settings {

	const PAGE = 'myroots-settings';

	public static function init(): void {
		add_action( 'admin_menu', [ __CLASS__, 'menu' ], 20 );
		add_action( 'admin_init', [ __CLASS__, 'register' ] );
		add_action( 'update_option_myroots3_slug', static function () {
			MyRoots_Site::rewrite();
			flush_rewrite_rules();
		} );
	}

	public static function menu(): void {
		$site = add_submenu_page( MyRoots_Editor::SLUG, __( 'Family website', 'myroots' ), __( 'Family website', 'myroots' ), 'manage_options', 'myroots-website', '__return_null' );
		add_submenu_page( MyRoots_Editor::SLUG, __( 'MyRoots settings', 'myroots' ), __( 'Settings', 'myroots' ), 'manage_options', self::PAGE, [ __CLASS__, 'page' ] );
		// "Family website" opens the public address
		add_action( 'load-' . $site, static function () {
			wp_safe_redirect( MyRoots_Site::url() );
			exit;
		} );
	}

	public static function register(): void {
		register_setting( self::PAGE, 'myroots3_slug', [ 'type' => 'string', 'sanitize_callback' => static function ( $v ) {
			$v = sanitize_title( (string) $v );
			return $v === '' ? '' : $v;
		} ] );
		register_setting( self::PAGE, 'myroots3_visibility', [ 'type' => 'string', 'sanitize_callback' => static function ( $v ) {
			return in_array( $v, [ 'public', 'users', 'off' ], true ) ? $v : 'public';
		} ] );
		register_setting( self::PAGE, 'myroots3_index', [ 'type' => 'boolean', 'sanitize_callback' => 'rest_sanitize_boolean' ] );
		register_setting( self::PAGE, 'myroots3_notify_suggestions', [ 'type' => 'boolean', 'sanitize_callback' => 'rest_sanitize_boolean', 'default' => true ] );
		register_setting( self::PAGE, 'myroots3_delete_on_uninstall', [ 'type' => 'boolean', 'sanitize_callback' => 'rest_sanitize_boolean' ] );
	}

	public static function page(): void {
		if ( ! current_user_can( 'manage_options' ) ) return;
		$vis = (string) get_option( 'myroots3_visibility', 'public' );
		?>
		<div class="wrap">
			<h1><?php esc_html_e( 'MyRoots settings', 'myroots' ); ?></h1>
			<p><?php esc_html_e( 'The family tree, its privacy, appearance and the content of the family website are set in the app itself.', 'myroots' ); ?>
				<a class="button button-primary" href="<?php echo esc_url( admin_url( 'admin.php?page=myroots' ) ); ?>"><?php esc_html_e( 'Open MyRoots', 'myroots' ); ?></a></p>
			<form method="post" action="options.php">
				<?php settings_fields( self::PAGE ); ?>
				<table class="form-table" role="presentation">
					<tr>
						<th scope="row"><label for="myroots3_slug"><?php esc_html_e( 'Address of the family website', 'myroots' ); ?></label></th>
						<td>
							<?php if ( get_option( 'permalink_structure' ) ) : ?>
								<code><?php echo esc_html( home_url( '/' ) ); ?></code><input name="myroots3_slug" id="myroots3_slug" type="text" class="regular-text" value="<?php echo esc_attr( MyRoots_Site::slug() ); ?>" /><code>/</code>
							<?php else : ?>
								<code><?php echo esc_html( MyRoots_Site::url() ); ?></code>
								<p class="description"><?php esc_html_e( 'Turn on pretty permalinks (Settings → Permalinks) to choose a nicer address.', 'myroots' ); ?></p>
							<?php endif; ?>
							<p><a href="<?php echo esc_url( MyRoots_Site::url() ); ?>" target="_blank" rel="noopener"><?php esc_html_e( 'Open the family website', 'myroots' ); ?></a></p>
						</td>
					</tr>
					<?php $myroots_trees = MyRoots_Trees::export(); if ( count( $myroots_trees ) > 1 ) : ?>
					<tr>
						<th scope="row"><?php esc_html_e( 'Family trees', 'myroots' ); ?></th>
						<td>
							<table class="widefat striped" style="max-width:760px">
								<tbody>
								<?php foreach ( $myroots_trees as $myroots_i => $myroots_t ) : ?>
									<tr>
										<td><strong><?php echo esc_html( $myroots_t['name'] !== '' ? $myroots_t['name'] : sprintf( /* translators: %d: number of the family tree */ __( 'Family tree %d', 'myroots' ), $myroots_i + 1 ) ); ?></strong></td>
										<td><a href="<?php echo esc_url( $myroots_t['url'] ); ?>" target="_blank" rel="noopener"><?php echo esc_html( $myroots_t['url'] ); ?></a></td>
										<td><code><?php echo esc_html( $myroots_t['slug'] === '' ? '[myroots]' : '[myroots tree="' . $myroots_t['slug'] . '"]' ); ?></code></td>
									</tr>
								<?php endforeach; ?>
								</tbody>
							</table>
							<p class="description"><?php esc_html_e( 'Trees are added, renamed and deleted in the app (Family trees). Each one is published separately.', 'myroots' ); ?></p>
						</td>
					</tr>
					<?php endif; ?>
					<tr>
						<th scope="row"><?php esc_html_e( 'Who can see it', 'myroots' ); ?></th>
						<td><fieldset>
							<label><input type="radio" name="myroots3_visibility" value="public" <?php checked( $vis, 'public' ); ?> /> <?php esc_html_e( 'Everybody', 'myroots' ); ?></label><br />
							<label><input type="radio" name="myroots3_visibility" value="users" <?php checked( $vis, 'users' ); ?> /> <?php esc_html_e( 'Only logged-in users of this site (family accounts)', 'myroots' ); ?></label><br />
							<label><input type="radio" name="myroots3_visibility" value="off" <?php checked( $vis, 'off' ); ?> /> <?php esc_html_e( 'Nobody (only editors of the tree)', 'myroots' ); ?></label>
						</fieldset>
						<p class="description"><?php esc_html_e( 'Living people are additionally protected by the privacy settings in the app (Family website).', 'myroots' ); ?></p></td>
					</tr>
					<tr>
						<th scope="row"><?php esc_html_e( 'Search engines', 'myroots' ); ?></th>
						<td><label><input type="checkbox" name="myroots3_index" value="1" <?php checked( (bool) get_option( 'myroots3_index', false ) ); ?> /> <?php esc_html_e( 'Allow search engines to index the family website', 'myroots' ); ?></label></td>
					</tr>
					<tr>
						<th scope="row"><?php esc_html_e( 'Suggestions from the family', 'myroots' ); ?></th>
						<td><label><input type="checkbox" name="myroots3_notify_suggestions" value="1" <?php checked( (bool) get_option( 'myroots3_notify_suggestions', true ) ); ?> /> <?php esc_html_e( 'Send me an e-mail when someone suggests a correction', 'myroots' ); ?></label>
						<p class="description"><?php esc_html_e( 'Suggestions appear in the app under Suggestions, the next time it is opened.', 'myroots' ); ?></p></td>
					</tr>
					<tr>
						<th scope="row"><?php esc_html_e( 'Removing the plugin', 'myroots' ); ?></th>
						<td><label><input type="checkbox" name="myroots3_delete_on_uninstall" value="1" <?php checked( (bool) get_option( 'myroots3_delete_on_uninstall', false ) ); ?> /> <?php esc_html_e( 'Delete the family tree, photos and the website when the plugin is deleted', 'myroots' ); ?></label>
						<p class="description"><?php esc_html_e( 'Make a backup first (in the app: Import and backup).', 'myroots' ); ?></p></td>
					</tr>
				</table>
				<?php submit_button(); ?>
			</form>
		</div>
		<?php
	}
}
