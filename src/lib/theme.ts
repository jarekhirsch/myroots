// Applies the "Appearance" settings (colors, font, size, corners, theme) as CSS variables.
import type { Settings } from './types';

type Ap = Settings['appearance'];

const FONTS: Record<string, string> = {
  system: "system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
  serif: "'Literata Variable', Georgia, 'Times New Roman', serif",
  humanist: "'Gill Sans', 'Gill Sans MT', Seravek, 'Trebuchet MS', sans-serif",
  rounded: "ui-rounded, 'SF Pro Rounded', 'Hiragino Maru Gothic ProN', Quicksand, 'Arial Rounded MT Bold', sans-serif",
  mono: "ui-monospace, 'SF Mono', Menlo, Consolas, monospace",
};

export const FONT_KEYS = Object.keys(FONTS);

const SIZES: Record<string, string> = { small: '14px', medium: '15px', large: '16.5px', xlarge: '18px' };
const RADII: Record<string, [string, string]> = { none: ['0px', '0px'], small: ['5px', '4px'], medium: ['10px', '6px'], large: ['16px', '10px'] };
const SHADOWS: Record<string, string> = {
  none: 'none',
  soft: '0 1px 2px rgba(20, 35, 50, 0.06), 0 2px 10px rgba(20, 35, 50, 0.05)',
  strong: '0 2px 4px rgba(20, 35, 50, 0.1), 0 6px 22px rgba(20, 35, 50, 0.12)',
};

const VARS = ['--ink', '--ink-2', '--ink-soft', '--heirloom', '--heirloom-soft', '--paper', '--text', '--line', '--font-ui', '--fs', '--radius', '--radius-sm', '--shadow'];

function hex(v: string): string | null {
  return /^#[0-9a-f]{6}$/i.test((v || '').trim()) ? v.trim() : null;
}

function mix(c: string, alpha: number): string {
  const n = parseInt(c.slice(1), 16);
  return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
}

export function applyAppearance(ap: Partial<Ap> | null | undefined) {
  const root = document.documentElement;
  for (const v of VARS) root.style.removeProperty(v);
  if (!ap) {
    root.removeAttribute('data-theme');
    return;
  }
  const theme = ap.theme === 'light' || ap.theme === 'dark' ? ap.theme : '';
  if (theme) root.setAttribute('data-theme', theme);
  else root.removeAttribute('data-theme');
  const dark = theme === 'dark' || (!theme && matchMedia('(prefers-color-scheme: dark)').matches);
  // Custom colors are tuned for the light theme; in dark mode only the accents apply.
  const primary = hex(ap.primary || '');
  const accent = hex(ap.accent || '');
  if (primary && !dark) {
    root.style.setProperty('--ink', primary);
    root.style.setProperty('--ink-2', `color-mix(in srgb, ${primary} 78%, #ffffff)`);
    root.style.setProperty('--ink-soft', mix(primary, 0.08));
  }
  if (accent) {
    root.style.setProperty('--heirloom', accent);
    root.style.setProperty('--heirloom-soft', mix(accent, 0.18));
  }
  if (!dark) {
    const bg = hex(ap.bg || '');
    const text = hex(ap.text || '');
    const border = hex(ap.border || '');
    if (bg) root.style.setProperty('--paper', bg);
    if (text) root.style.setProperty('--text', text);
    if (border) root.style.setProperty('--line', border);
  }
  if (ap.font && FONTS[ap.font]) root.style.setProperty('--font-ui', FONTS[ap.font]);
  if (ap.font_size && SIZES[ap.font_size]) root.style.setProperty('--fs', SIZES[ap.font_size]);
  if (ap.radius && RADII[ap.radius]) {
    root.style.setProperty('--radius', RADII[ap.radius][0]);
    root.style.setProperty('--radius-sm', RADII[ap.radius][1]);
  }
  if (ap.shadow && SHADOWS[ap.shadow] && !dark) root.style.setProperty('--shadow', SHADOWS[ap.shadow]);
}
