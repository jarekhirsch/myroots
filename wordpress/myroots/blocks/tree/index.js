/* MyRoots block: the family website of MyRoots inside a page (an iframe). */
( function ( wp ) {
	var el = wp.element.createElement;
	var __ = wp.i18n.__;
	var be = wp.blockEditor;
	var c = wp.components;
	var VIEWS = [
		[ '/', __( 'Home', 'myroots' ) ],
		[ '/people', __( 'People', 'myroots' ) ],
		[ '/tree', __( 'Family tree', 'myroots' ) ],
		[ '/map', __( 'Map', 'myroots' ) ],
		[ '/lifeline', __( 'Lifeline', 'myroots' ) ],
		[ '/gallery', __( 'Gallery', 'myroots' ) ],
		[ '/events', __( 'Events', 'myroots' ) ],
		[ '/stats', __( 'Statistics', 'myroots' ) ],
		[ '/pets', __( 'Pets', 'myroots' ) ],
	];
	wp.blocks.registerBlockType( 'myroots/tree', {
		edit: function ( props ) {
			var a = props.attributes;
			return el(
				'div',
				be.useBlockProps(),
				el(
					be.InspectorControls,
					null,
					el(
						c.PanelBody,
						{ title: __( 'Settings', 'myroots' ) },
						( window.MYROOTS_TREES || [] ).length > 1 &&
							el( c.SelectControl, {
								label: __( 'Family tree', 'myroots' ),
								value: a.tree,
								options: window.MYROOTS_TREES.map( function ( t ) { return { value: t.slug, label: t.name }; } ),
								onChange: function ( v ) { props.setAttributes( { tree: v } ); },
							} ),
						el( c.SelectControl, {
							label: __( 'Start view', 'myroots' ),
							value: VIEWS.some( function ( v ) { return v[ 0 ] === a.view; } ) ? a.view : '',
							options: VIEWS.map( function ( v ) { return { value: v[ 0 ], label: v[ 1 ] }; } ).concat( [ { value: '', label: __( 'Other (below)', 'myroots' ) } ] ),
							onChange: function ( v ) { if ( v ) props.setAttributes( { view: v } ); },
						} ),
						el( c.TextControl, {
							label: __( 'View (e.g. /person/12)', 'myroots' ),
							value: a.view,
							onChange: function ( v ) { props.setAttributes( { view: v } ); },
						} ),
						el( c.TextControl, {
							label: __( 'Height (e.g. 85vh or 700px)', 'myroots' ),
							value: a.height,
							onChange: function ( v ) { props.setAttributes( { height: v } ); },
						} )
					)
				),
				el( wp.serverSideRender, { block: 'myroots/tree', attributes: a } )
			);
		},
		save: function () {
			return null;
		},
	} );
} )( window.wp );
