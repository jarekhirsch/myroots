<?php
/**
 * Removing MyRoots 3.
 *
 * The family tree is deleted only when "Delete the family tree, photos and the
 * website when the plugin is deleted" is turned on in MyRoots → Settings.
 * Data of MyRoots 2.x (posts of people, pets and sources) is never touched.
 */

if ( ! defined( 'WP_UNINSTALL_PLUGIN' ) ) exit;

function myroots3_uninstall_site() {
	global $wpdb;
	if ( ! get_option( 'myroots3_delete_on_uninstall' ) ) return;
	$trees = get_option( 'myroots3_trees' );
	$ids   = is_array( $trees ) && $trees ? array_keys( $trees ) : [ 'default' ];
	if ( ! in_array( 'default', $ids, true ) ) $ids[] = 'default';
	$up   = wp_upload_dir( null, false );
	$base = trailingslashit( $up['basedir'] ) . 'myroots';
	$sec  = (string) get_option( 'myroots3_secret', '' );
	$opts = [ 'myroots3_db', 'myroots3_secret', 'myroots3_slug', 'myroots3_visibility', 'myroots3_index', 'myroots3_notify_suggestions', 'myroots3_delete_on_uninstall', 'myroots3_rules', 'myroots3_trees' ];
	foreach ( $ids as $id ) {
		$id  = preg_replace( '/[^a-z0-9]/', '', strtolower( (string) $id ) );
		$suf = $id === 'default' ? '' : '_' . $id;
		$wpdb->query( $wpdb->prepare( 'DROP TABLE IF EXISTS %i', $wpdb->prefix . 'myroots_records' . $suf ) ); // phpcs:ignore WordPress.DB.DirectDatabaseQuery, WordPress.DB.DirectDatabaseQuery.SchemaChange
		if ( $sec !== '' ) {
			foreach ( [ 'files', 'site' ] as $what ) {
				myroots3_rmdir( $base . '/' . $what . '-' . $sec . ( $suf === '' ? '' : '-' . $id ) );
			}
		}
		foreach ( [ 'myroots3_gen', 'myroots3_changed', 'myroots3_site', 'myroots3_auto_publish', 'myroots3_inbox', 'myroots3_weather_decided' ] as $o ) {
			$opts[] = $o . $suf;
		}
	}
	foreach ( $opts as $o ) {
		delete_option( $o );
	}
	delete_metadata( 'user', 0, 'myroots3_tree', '', true );
}

function myroots3_rmdir( $dir ) {
	if ( ! is_dir( $dir ) ) return;
	foreach ( (array) scandir( $dir ) as $f ) {
		if ( $f === '.' || $f === '..' ) continue;
		$p = $dir . '/' . $f;
		if ( is_dir( $p ) ) myroots3_rmdir( $p );
		else wp_delete_file( $p );
	}
	@rmdir( $dir ); // phpcs:ignore
}

if ( is_multisite() ) {
	foreach ( get_sites( [ 'fields' => 'ids' ] ) as $myroots3_blog ) {
		switch_to_blog( $myroots3_blog );
		myroots3_uninstall_site();
		restore_current_blog();
	}
} else {
	myroots3_uninstall_site();
}
