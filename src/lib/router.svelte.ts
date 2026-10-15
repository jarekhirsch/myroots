// Tiny hash router (works in Tauri, the PWA and the static family website).
export const route = $state({ path: '/', parts: [] as string[], query: {} as Record<string, string> });

/** The views visited in this window, oldest first (for the Back button). */
export const trail = $state({ stack: [] as string[] });
const KEY = 'myroots-trail';
let replacing = false;

function current() {
  return location.hash.replace(/^#/, '') || '/';
}

function parse() {
  const h = current();
  const [path, qs] = h.split('?');
  route.path = path || '/';
  route.parts = route.path.split('/').filter(Boolean).map(decodeURIComponent);
  route.query = Object.fromEntries(new URLSearchParams(qs || ''));
  remember(h);
}

function remember(h: string) {
  const s = trail.stack;
  if (replacing && s.length) s[s.length - 1] = h;
  else if (s.length > 1 && s[s.length - 2] === h) s.pop(); // went back
  else if (s[s.length - 1] !== h) s.push(h);
  if (s.length > 60) s.splice(0, s.length - 60);
  replacing = false;
  try {
    sessionStorage.setItem(KEY, JSON.stringify(s));
  } catch {
    /* private mode: the trail lives in memory only */
  }
}

export function initRouter() {
  try {
    const saved = JSON.parse(sessionStorage.getItem(KEY) || '[]');
    if (Array.isArray(saved) && saved[saved.length - 1] === current()) trail.stack = saved;
  } catch {
    /* ignore */
  }
  parse();
  window.addEventListener('hashchange', () => {
    parse();
    window.scrollTo({ top: 0 });
  });
}

export function go(path: string, replace = false) {
  const h = '#' + path;
  if (replace) {
    replacing = true;
    history.replaceState(null, '', h);
    parse();
  } else location.hash = path;
}

/** The natural parent of a view, for when there is nothing to go back to. */
export function parentOf(path: string): string | null {
  const p = path.split('?')[0].split('/').filter(Boolean);
  const [a, b, c] = p;
  if (!a) return null;
  if ((a === 'person' || a === 'pet' || a === 'source') && b) {
    const list = a === 'person' ? '/people' : a === 'pet' ? '/pets' : '/sources';
    if (b === 'new') return list;
    return c ? `/${a}/${b}` : list;
  }
  if ((a === 'lifeline' || a === 'tree' || a === 'map') && b) return `/person/${b}`;
  return null;
}

/** Is there somewhere to go back to from this view? */
export function canGoBack(): boolean {
  return parentOf(route.path) != null || (trail.stack.length > 1 && Object.keys(route.query).length > 0);
}

export function back(fallback = '/') {
  if (trail.stack.length > 1) history.back();
  // opened directly: go up to the parent, in place (so Back never loops)
  else go(parentOf(current()) || fallback, true);
}

export function href(path: string) {
  return '#' + path;
}
