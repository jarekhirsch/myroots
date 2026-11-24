<?php
/**
 * Plugin Name:       MyRoots
 * Plugin URI:        https://myroots.pl/
 * Description:       Family tree for WordPress, with the very same interface and engine as the MyRoots 3 app for Mac, Windows, Linux, phones and the browser. Edit the tree in the admin panel and publish a read-only family website for your relatives.
 * Version:           3.2.0
 * Author:            Jarek Hirsch
 * Author URI:        https://hirsch.fi/
 * License:           GPL-3.0-or-later
 * License URI:       https://www.gnu.org/licenses/gpl-3.0.html
 * Requires at least: 6.5
 * Requires PHP:      7.4
 * Text Domain:       myroots
 * Domain Path:       /languages
 */

/*
 * Author contact: Jarek Hirsch <jarekhirsz.biz@gmail.com>
 *
 * Architecture: the editor in wp-admin is the MyRoots 3 app (Svelte interface +
 * the Rust engine compiled to WebAssembly). WordPress stores what the engine
 * produces: records in one database table, photos and recordings in the uploads
 * folder. Visitors get the read-only family website (the same viewer as the
 * exported website of the app), published by the editor with privacy applied.
 */

if ( ! defined( 'ABSPATH' ) ) exit;

define( 'MYROOTS_VERSION', '3.2.0' );
define( 'MYROOTS_FILE', __FILE__ );
define( 'MYROOTS_PATH', plugin_dir_path( __FILE__ ) );
define( 'MYROOTS_URL', plugin_dir_url( __FILE__ ) );
define( 'MYROOTS_CAP', 'myroots_manage' );

require_once MYROOTS_PATH . 'includes/class-trees.php';
require_once MYROOTS_PATH . 'includes/class-store.php';
require_once MYROOTS_PATH . 'includes/class-rest.php';
require_once MYROOTS_PATH . 'includes/class-editor.php';
require_once MYROOTS_PATH . 'includes/class-site.php';
require_once MYROOTS_PATH . 'includes/class-pwa.php';
require_once MYROOTS_PATH . 'includes/class-legacy.php';
require_once MYROOTS_PATH . 'includes/class-settings.php';

/** Can the current user edit the tree? */
function myroots_can_edit(): bool {
	$ok = current_user_can( MYROOTS_CAP ) || current_user_can( 'manage_options' );
	return (bool) apply_filters( 'myroots_can_edit', $ok );
}

add_action( 'plugins_loaded', static function () {
	load_plugin_textdomain( 'myroots', false, dirname( plugin_basename( __FILE__ ) ) . '/languages' );
	MyRoots_Store::maybe_upgrade();
	MyRoots_Rest::init();
	MyRoots_Editor::init();
	MyRoots_Site::init();
	MyRoots_Settings::init();
} );

/** What the server must offer; checked before anything is used (activation and admin notice). */
function myroots_requirements(): array {
	$missing = [];
	if ( version_compare( PHP_VERSION, '7.4', '<' ) ) {
		/* translators: %s: PHP version */
		$missing[] = sprintf( __( 'MyRoots needs PHP 7.4 or newer (this server has %s).', 'myroots' ), PHP_VERSION );
	}
	if ( version_compare( get_bloginfo( 'version' ), '6.5', '<' ) ) {
		$missing[] = __( 'MyRoots needs WordPress 6.5 or newer.', 'myroots' );
	}
	if ( ! function_exists( 'json_decode' ) ) {
		$missing[] = __( 'MyRoots needs the PHP JSON extension.', 'myroots' );
	}
	return $missing;
}

add_action( 'admin_notices', static function () {
	if ( ! current_user_can( 'activate_plugins' ) ) return;
	$problems = myroots_requirements();
	if ( ! MyRoots_Store::writable() ) {
		$problems[] = __( 'MyRoots cannot write to the uploads folder (wp-content/uploads), so photos and the family website cannot be saved. Please make the folder writable.', 'myroots' );
	}
	foreach ( $problems as $p ) {
		printf( '<div class="notice notice-error"><p>%s</p></div>', esc_html( $p ) );
	}
} );

register_activation_hook( __FILE__, static function () {
	$missing = myroots_requirements();
	if ( $missing ) {
		deactivate_plugins( plugin_basename( MYROOTS_FILE ) );
		wp_die( esc_html( implode( ' ', $missing ) ), esc_html__( 'Plugin could not be activated', 'myroots' ), [ 'back_link' => true ] );
	}
	MyRoots_Store::install();
	// The role and the capability of the 2.x plugin are kept, so its editors keep working.
	$admin = get_role( 'administrator' );
	if ( $admin ) $admin->add_cap( MYROOTS_CAP );
	if ( ! get_role( 'myroots_genealogy_editor' ) ) {
		add_role( 'myroots_genealogy_editor', __( 'Genealogy Editor', 'myroots' ), [ 'read' => true, MYROOTS_CAP => true ] );
	}
	MyRoots_Site::rewrite();
	flush_rewrite_rules();
} );

register_deactivation_hook( __FILE__, static function () {
	flush_rewrite_rules();
} );
