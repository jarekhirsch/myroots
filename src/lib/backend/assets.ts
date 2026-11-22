declare const __VIEWER__: boolean;

// Where the static files of the app live. Normally next to the page ("./"),
// inside WordPress in the plugin folder (the page itself is wp-admin or the
// public tree address, so relative paths would point to the wrong place).

export function assetUrl(path: string): string {
  const w = window as any;
  const base: string = w.MYROOTS_WP?.assets || w.MYROOTS_ASSETS || './';
  return base + path.replace(/^\.\//, '');
}

/**
 * Read-only mode: the exported family website (its own build) or, inside
 * WordPress, the app itself shown to visitors through the [myroots] shortcode.
 */
export function isViewerMode(): boolean {
  return __VIEWER__ || (typeof window !== 'undefined' && !!(window as any).MYROOTS_WP_VIEW);
}

/** Configuration printed by the WordPress plugin (editor in wp-admin). */
export interface WpConfig {
  rest: string;
  nonce: string;
  assets: string;
  home: string;
  admin: string;
  site_url: string;
  site_name: string;
  gen: number;
  legacy: number;
  can_publish: boolean;
  auto_publish: boolean;
  published: number;
  user: string;
  ajax: string;
  lang: string;
  weather_decided: boolean;
  /** The family tree open in the editor and its address segment ('' = main tree). */
  tree?: string;
  tree_slug?: string;
  /** "server": MyRoots Server (own PHP application) instead of WordPress. */
  host?: 'wp' | 'server';
  back_label?: string;
  logout?: string;
}

/** The app runs on MyRoots Server (not inside WordPress). */
export function isServerHost(): boolean {
  return (window as any).MYROOTS_WP?.host === 'server';
}

export function wpConfig(): WpConfig | null {
  return ((window as any).MYROOTS_WP as WpConfig) || null;
}

/** Link back to the WordPress site (editor and public tree inside WordPress). */
export function wpBack(): { url: string; label: string } | null {
  const w = window as any;
  if (w.MYROOTS_WP) return { url: w.MYROOTS_WP.admin, label: w.MYROOTS_WP.back_label || w.MYROOTS_WP.site_name || 'WordPress' };
  if (w.MYROOTS_WP_VIEW?.home) return { url: w.MYROOTS_WP_VIEW.home, label: w.MYROOTS_WP_VIEW.site_name || '' };
  return null;
}
