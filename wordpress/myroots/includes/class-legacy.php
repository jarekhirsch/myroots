<?php
/**
 * MyRoots_Legacy: data of MyRoots 2.x on the same site.
 *
 * Version 2 kept people, pets and sources as posts with meta. This class
 * produces the same JSON backup that 2.x wrote (MyRoots → Backup), read
 * straight from the database, so the app imports it with its WordPress
 * importer and then downloads the photos from the media library of this site.
 * The old posts are never changed or removed.
 */

if ( ! defined( 'ABSPATH' ) ) exit;

class MyRoots_Legacy {

	const PERSON = 'myroots_person';
	const PET    = 'myroots_pet';
	const SOURCE = 'myroots_source';

	private static function posts( string $type ): array {
		global $wpdb;
		// phpcs:ignore WordPress.DB.DirectDatabaseQuery
		return (array) $wpdb->get_results( $wpdb->prepare(
			"SELECT ID, post_title, post_status, post_content FROM {$wpdb->posts} WHERE post_type = %s AND post_status NOT IN ('trash','auto-draft','inherit') ORDER BY ID",
			$type
		) );
	}

	public static function count(): int {
		global $wpdb;
		// phpcs:ignore WordPress.DB.DirectDatabaseQuery
		return (int) $wpdb->get_var( $wpdb->prepare(
			"SELECT COUNT(*) FROM {$wpdb->posts} WHERE post_type = %s AND post_status NOT IN ('trash','auto-draft','inherit')",
			self::PERSON
		) );
	}

	private static function meta( int $id, string $key ): array {
		$v = get_post_meta( $id, $key, true );
		return is_array( $v ) ? $v : [];
	}

	public static function export(): array {
		$persons = [];
		foreach ( self::posts( self::PERSON ) as $p ) {
			$persons[] = [
				'id'        => (int) $p->ID,
				'title'     => $p->post_title,
				'status'    => $p->post_status,
				'content'   => $p->post_content,
				'thumbnail' => (int) get_post_thumbnail_id( (int) $p->ID ),
				'data'      => (object) self::meta( (int) $p->ID, '_myroots' ),
			];
		}
		$pets = [];
		foreach ( self::posts( self::PET ) as $p ) {
			$pets[] = [
				'id'        => (int) $p->ID,
				'title'     => $p->post_title,
				'status'    => $p->post_status,
				'content'   => $p->post_content,
				'thumbnail' => (int) get_post_thumbnail_id( (int) $p->ID ),
				'data'      => (object) self::meta( (int) $p->ID, '_myroots_pet' ),
			];
		}
		$sources = [];
		foreach ( self::posts( self::SOURCE ) as $s ) {
			$linked    = array_values( array_unique( array_filter( array_map( 'intval', (array) get_post_meta( (int) $s->ID, '_myroots_linked_person', false ) ) ) ) );
			$sources[] = [
				'id'        => (int) $s->ID,
				'title'     => $s->post_title,
				'status'    => $s->post_status,
				'thumbnail' => (int) get_post_thumbnail_id( (int) $s->ID ),
				'meta'      => (object) self::meta( (int) $s->ID, '_myroots_source' ),
				'persons'   => $linked,
			];
		}
		return [
			'format'   => 1,
			'plugin'   => (string) get_option( 'myroots_version', '2.x' ),
			'site'     => home_url(),
			'exported' => gmdate( 'c' ),
			'persons'  => $persons,
			'pets'     => $pets,
			'sources'  => $sources,
			'options'  => [
				'clans'         => (array) get_option( 'myroots_clans', [] ),
				'custom_schema' => (string) get_option( 'myroots_custom_schema', '[]' ),
			],
		];
	}
}
