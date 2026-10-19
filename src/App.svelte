<script lang="ts">
  import { app, call, refreshInfo } from './lib/api.svelte';
  import { route, go, back, canGoBack } from './lib/router.svelte';
  import { ui, toast } from './lib/ui.svelte';
  import { t, lang } from './lib/i18n/index.svelte';
  import { NAV, TABBAR, DOCK, visible, isActive } from './lib/nav';
  import { backend } from './lib/backend';
  import Modal from './lib/components/Modal.svelte';
  import Lightbox from './lib/components/Lightbox.svelte';
  import QuickSearch from './lib/components/QuickSearch.svelte';
  import BackButton from './lib/components/BackButton.svelte';
  import AboutDialog from './lib/components/AboutDialog.svelte';
  import { AUTHOR, YEAR, authorUrl } from './lib/credits';
  import MenuIcon from '@lucide/svelte/icons/menu';
  import More from '@lucide/svelte/icons/ellipsis';
  import X from '@lucide/svelte/icons/x';

  import Home from './views/Home.svelte';
  import People from './views/People.svelte';
  import PersonView from './views/PersonView.svelte';
  import PersonEdit from './views/PersonEdit.svelte';
  import PersonHistory from './views/PersonHistory.svelte';
  import AgingView from './views/AgingView.svelte';
  import LifelineView from './views/LifelineView.svelte';
  import TreeView from './views/TreeView.svelte';
  import MapView from './views/MapView.svelte';
  import Graves from './views/Graves.svelte';
  import Poster from './views/Poster.svelte';
  import Gallery from './views/Gallery.svelte';
  import Events from './views/Events.svelte';
  import Pets from './views/Pets.svelte';
  import PetView from './views/PetView.svelte';
  import PetEdit from './views/PetEdit.svelte';
  import Sources from './views/Sources.svelte';
  import SourceView from './views/SourceView.svelte';
  import SourceEdit from './views/SourceEdit.svelte';
  import Stats from './views/Stats.svelte';
  import Kinship from './views/Kinship.svelte';
  import BookView from './views/BookView.svelte';
  import Health from './views/Health.svelte';
  import Duplicates from './views/Duplicates.svelte';
  import Suggestions from './views/Suggestions.svelte';
  import Clans from './views/Clans.svelte';
  import Places from './views/Places.svelte';
  import SettingsView from './views/SettingsView.svelte';
  import DataView from './views/DataView.svelte';
  import Website from './views/Website.svelte';
  import NotFound from './views/NotFound.svelte';

  const viewer = $derived(!!app.info?.viewer);
  const sections = $derived(app.info?.sections);
  // A new, empty tree shows only what helps to start.
  const START_ONLY = ['/', '/data', '/settings'];
  const groups = $derived(
    NAV.map((g) => ({ ...g, items: g.items.filter((i) => visible(i, viewer, sections) && (viewer || !app.info?.empty || START_ONLY.includes(i.path))) })).filter((g) => g.items.length),
  );
  $effect(() => {
    void app.version;
    if (app.info?.empty && !viewer) call<any>('app.info').then((i) => { if (!i.empty) app.info = i; });
  });
  const dock = $derived(DOCK.map((p) => groups.flatMap((g) => g.items).find((i) => i.path === p)).filter((x) => !!x));
  // The page the menu panel would highlight when it is not in the dock.
  const inMenu = $derived(!dock.some((i) => isActive(i!.path, route.path)) && route.path !== '/');
  const tabs = $derived(TABBAR.map((p) => groups.flatMap((g) => g.items).find((i) => i.path === p)).filter((x) => !!x));
  const title = $derived(viewer ? app.info?.site?.title || app.info?.tree_name || 'MyRoots' : app.info?.tree_name || 'MyRoots');

  let newSuggestions = $state(0);
  $effect(() => {
    void app.version;
    if (viewer) return;
    call<any[]>('suggestions.list')
      .then((v) => (newSuggestions = v.filter((s) => s.status === 'new').length))
      .catch(() => {});
  });

  $effect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && ui.menuOpen) ui.menuOpen = false;
    };
    window.addEventListener('keydown', k);
    return () => window.removeEventListener('keydown', k);
  });

  let about = $state(false);
  // The tree and the map have their own toolbars (with their own Back button).
  const fullScreen = $derived(route.parts[0] === 'tree' || route.parts[0] === 'map');
  const withBack = $derived((void route.path, void route.query, !fullScreen && canGoBack()));

  // Back with the keyboard and the mouse (the desktop app has no browser buttons)
  $effect(() => {
    const typing = (e: Event) => {
      const el = e.target as HTMLElement | null;
      return !!el && (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName));
    };
    const key = (e: KeyboardEvent) => {
      const combo = (e.altKey && e.key === 'ArrowLeft') || ((e.metaKey || e.ctrlKey) && e.key === '[');
      if (combo && !typing(e) && !ui.menuOpen) {
        e.preventDefault();
        back();
      }
    };
    const mouse = (e: MouseEvent) => {
      if (e.button === 3) {
        e.preventDefault();
        back();
      }
    };
    window.addEventListener('keydown', key);
    window.addEventListener('mouseup', mouse);
    return () => {
      window.removeEventListener('keydown', key);
      window.removeEventListener('mouseup', mouse);
    };
  });

  // close the menu panel after navigation
  $effect(() => {
    void route.path;
    ui.menuOpen = false;
  });

  const p = $derived(route.parts);
  const editorOnly = $derived(['sources', 'source', 'health', 'duplicates', 'suggestions', 'clans', 'places', 'settings', 'data', 'website'].includes(p[0] || ''));

  let promptValue = $state('');
  $effect(() => {
    if (ui.prompt) promptValue = ui.prompt.value;
  });

  function closeConfirm(v: boolean) {
    ui.confirm?.resolve(v);
    ui.confirm = null;
  }
  function closePrompt(v: string | null) {
    ui.prompt?.resolve(v);
    ui.prompt = null;
  }

  // Remember the browser storage for the PWA; flush pending writes before leaving.
  $effect(() => {
    const h = () => {
      backend.flush?.();
    };
    window.addEventListener('pagehide', h);
    return () => window.removeEventListener('pagehide', h);
  });
</script>

{#key lang()}
  <div class="shell" class:viewer>
    <nav class="dock no-print" aria-label={t('Menu')}>
      <a href="#/" class="dock-logo" aria-label={title}><img src="./icon.svg" alt="" width="40" height="40" /></a>
      {#each dock as item (item!.path)}
        {@const Icon = item!.icon}
        <a href="#{item!.path}" class="dk" class:on={isActive(item!.path, route.path)} aria-current={isActive(item!.path, route.path) ? 'page' : undefined}>
          <span class="dk-ic"><Icon size={21} strokeWidth={1.8} /></span>
          <span class="dk-l">{(item!.short || item!.label)()}</span>
        </a>
      {/each}
      <button class="dk more" class:on={inMenu || ui.menuOpen} onclick={() => (ui.menuOpen = !ui.menuOpen)} aria-expanded={ui.menuOpen}>
        <span class="dk-ic"><MenuIcon size={21} strokeWidth={1.8} />{#if newSuggestions && !viewer}<span class="dot" aria-hidden="true"></span>{/if}</span>
        <span class="dk-l">{t('More')}</span>
      </button>
    </nav>
    <aside class="rail no-print" class:open={ui.menuOpen} aria-label={t('Menu')}>
      <div class="brand">
        <a href="#/" class="logo" aria-label="MyRoots">
          <img src="./icon.svg" alt="" width="36" height="36" />
          <span class="nm">
            <span class="name">{title}</span>
            {#if viewer && app.info?.site?.subtitle}<span class="sub">{app.info.site.subtitle}</span>{:else if !viewer && title !== 'MyRoots'}<span class="sub">MyRoots</span>{/if}
          </span>
        </a>
        <button class="btn ghost icon close-menu" onclick={() => (ui.menuOpen = false)} aria-label={t('Close')}><X size={18} /></button>
      </div>
      {#if !app.info?.empty && (!viewer || sections?.list !== false)}
        <div class="qs"><QuickSearch /></div>
      {/if}
      <nav>
        {#each groups as g (g.label())}
          <div class="grp">
            <div class="glabel">{g.label()}</div>
            {#each g.items as item (item.path)}
              {@const Icon = item.icon}
              <a href="#{item.path}" class="item" class:on={isActive(item.path, route.path)} aria-current={isActive(item.path, route.path) ? 'page' : undefined}>
                <Icon size={18} strokeWidth={1.8} />
                <span>{item.label()}</span>
                {#if item.badge === 'suggestions' && newSuggestions}<span class="badge">{newSuggestions}</span>{/if}
              </a>
            {/each}
          </div>
        {/each}
      </nav>
      <div class="foot small faint">
        <button type="button" class="about-link" onclick={() => ((about = true), (ui.menuOpen = false))}>
          {#if viewer}{t('Made with MyRoots')}{:else}MyRoots {app.info?.version}{/if}
        </button>
        <span class="credit">© {YEAR} <a href={authorUrl(viewer ? 'family-website' : 'app', 'menu')} target="_blank" rel="noopener">{AUTHOR}</a> · GPL-3.0+</span>
      </div>
    </aside>
    {#if ui.menuOpen}<div class="scrim no-print" role="presentation" onclick={() => (ui.menuOpen = false)}></div>{/if}

    <header class="topbar no-print">
      <button class="btn ghost icon" onclick={() => (ui.menuOpen = true)} aria-label={t('Menu')}><MenuIcon size={20} /></button>
      <a href="#/" class="tb-title name">{title}</a>
      <div class="tb-search"><QuickSearch compact /></div>
    </header>

    <main class="main" class:with-back={withBack}>
      {#if withBack}<div class="backbar"><BackButton /></div>{/if}
      {#if viewer && editorOnly}
        <NotFound />
      {:else if p.length === 0}
        <Home />
      {:else if p[0] === 'people'}
        <People />
      {:else if p[0] === 'person' && p[1] === 'new'}
        <PersonEdit id={null} />
      {:else if p[0] === 'person' && p[2] === 'edit' && !viewer}
        <PersonEdit id={Number(p[1])} />
      {:else if p[0] === 'person' && p[2] === 'history' && !viewer}
        <PersonHistory id={Number(p[1])} />
      {:else if p[0] === 'person' && p[2] === 'years'}
        <AgingView id={Number(p[1])} />
      {:else if p[0] === 'person' && p[1]}
        <PersonView id={Number(p[1])} />
      {:else if p[0] === 'lifeline'}
        <LifelineView id={p[1] ? Number(p[1]) : null} />
      {:else if p[0] === 'tree'}
        <TreeView id={p[1] ? Number(p[1]) : null} />
      {:else if p[0] === 'map'}
        <MapView person={p[1] ? Number(p[1]) : null} />
      {:else if p[0] === 'gallery'}
        <Gallery />
      {:else if p[0] === 'graves'}
        <Graves />
      {:else if p[0] === 'events'}
        <Events />
      {:else if p[0] === 'pets'}
        <Pets />
      {:else if p[0] === 'pet' && p[1] === 'new' && !viewer}
        <PetEdit id={null} />
      {:else if p[0] === 'pet' && p[2] === 'edit' && !viewer}
        <PetEdit id={Number(p[1])} />
      {:else if p[0] === 'pet' && p[1]}
        <PetView id={Number(p[1])} />
      {:else if p[0] === 'sources'}
        <Sources />
      {:else if p[0] === 'source' && p[1] === 'new'}
        <SourceEdit id={null} />
      {:else if p[0] === 'source' && p[2] === 'edit'}
        <SourceEdit id={Number(p[1])} />
      {:else if p[0] === 'source' && p[1]}
        <SourceView id={Number(p[1])} />
      {:else if p[0] === 'stats'}
        <Stats />
      {:else if p[0] === 'kinship'}
        <Kinship />
      {:else if p[0] === 'poster'}
        <Poster id={p[1] ? Number(p[1]) : null} />
      {:else if p[0] === 'book'}
        <BookView />
      {:else if p[0] === 'health'}
        <Health />
      {:else if p[0] === 'duplicates'}
        <Duplicates />
      {:else if p[0] === 'suggestions'}
        <Suggestions />
      {:else if p[0] === 'clans'}
        <Clans />
      {:else if p[0] === 'places'}
        <Places />
      {:else if p[0] === 'settings'}
        <SettingsView />
      {:else if p[0] === 'data'}
        <DataView />
      {:else if p[0] === 'website'}
        <Website />
      {:else}
        <NotFound />
      {/if}
    </main>

    <div class="print-credit" aria-hidden="true">© MyRoots · {AUTHOR}</div>
    <nav class="tabbar no-print" aria-label={t('Menu')}>
      {#each tabs as item (item!.path)}
        {@const Icon = item!.icon}
        <a href="#{item!.path}" class:on={isActive(item!.path, route.path)}>
          <Icon size={21} strokeWidth={1.8} />
          <span>{(item!.short || item!.label)()}</span>
        </a>
      {/each}
      <button onclick={() => (ui.menuOpen = true)}>
        <More size={21} strokeWidth={1.8} />
        <span>{t('More')}</span>
      </button>
    </nav>
  </div>

  <div class="toasts" aria-live="polite">
    {#each ui.toasts as tt (tt.id)}
      <div class="toast {tt.kind}">{tt.text}</div>
    {/each}
  </div>

  {#if ui.confirm}
    <Modal title={ui.confirm.title} onclose={() => closeConfirm(false)}>
      {#if ui.confirm.body}<p class="pre">{ui.confirm.body}</p>{/if}
      {#snippet footer()}
        <button class="btn" onclick={() => closeConfirm(false)}>{t('Cancel')}</button>
        <button class="btn {ui.confirm!.danger ? 'danger' : 'primary'}" onclick={() => closeConfirm(true)}>{ui.confirm!.ok || t('OK')}</button>
      {/snippet}
    </Modal>
  {/if}

  {#if ui.prompt}
    <Modal title={ui.prompt.title} onclose={() => closePrompt(null)}>
      <form id="prompt-form" onsubmit={(e) => { e.preventDefault(); closePrompt(promptValue); }}>
        <label class="field">
          {#if ui.prompt.label}<span class="lbl">{ui.prompt.label}</span>{/if}
          <input type="text" bind:value={promptValue} />
        </label>
      </form>
      {#snippet footer()}
        <button class="btn" onclick={() => closePrompt(null)}>{t('Cancel')}</button>
        <button class="btn primary" type="submit" form="prompt-form">{ui.prompt!.ok || t('OK')}</button>
      {/snippet}
    </Modal>
  {/if}

  {#if ui.lightbox}
    <Lightbox />
  {/if}
{#if about}<AboutDialog onclose={() => (about = false)} />{/if}
{/key}

<style>
  .shell {
    min-height: 100vh;
    padding-inline-start: var(--dock);
  }
  /* narrow dock with the main places; everything else is in the menu panel */
  .dock {
    position: fixed;
    inset-block: 0;
    inset-inline-start: 0;
    width: var(--dock);
    z-index: 70;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 16px 0;
    background: var(--paper);
    box-sizing: border-box;
  }
  .dock-logo {
    margin-bottom: 14px;
    border-radius: 14px;
  }
  .dock-logo img {
    display: block;
  }
  .dk {
    width: 72px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 4px 2px 6px;
    border: 0;
    background: none;
    font: inherit;
    color: var(--muted);
    text-decoration: none;
    cursor: pointer;
    border-radius: 16px;
  }
  .dk-ic {
    position: relative;
    width: 48px;
    height: 34px;
    border-radius: 999px;
    display: grid;
    place-items: center;
    transition: background 0.15s;
  }
  .dk:hover .dk-ic {
    background: var(--surface-2);
  }
  .dk.on {
    color: var(--ink);
  }
  .dk.on .dk-ic {
    background: var(--ink);
    color: var(--surface);
  }
  .dk-l {
    font-size: 0.72rem;
    font-weight: 550;
    line-height: 1.15;
    text-align: center;
    max-width: 100%;
    overflow-wrap: anywhere;
  }
  .dk.on .dk-l {
    font-weight: 680;
  }
  .dk.more {
    margin-top: auto;
  }
  .dot {
    position: absolute;
    top: 3px;
    inset-inline-end: 8px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--heirloom);
    box-shadow: 0 0 0 2px var(--paper);
  }
  .rail {
    position: fixed;
    inset-block: 0;
    inset-inline-start: var(--dock);
    width: var(--rail);
    background: var(--surface);
    box-shadow: var(--shadow-pop);
    transform: translateX(calc(-100% - var(--dock)));
    transition: transform 0.2s ease, visibility 0s linear 0.2s;
    visibility: hidden;
    display: flex;
    flex-direction: column;
    z-index: 60;
    overflow-y: auto;
    overscroll-behavior: contain;
  }
  .brand {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 22px 16px 14px 20px;
  }
  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
    color: var(--text);
    min-width: 0;
  }
  .logo img {
    flex-shrink: 0;
  }
  .logo .nm {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.2;
  }
  .logo .name {
    font-size: 1.08rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .logo .sub {
    font-size: 0.78rem;
    color: var(--faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .close-menu {
    display: none;
  }
  .qs {
    padding: 2px 16px 6px;
  }
  .rail nav {
    flex: 1;
    padding: 2px 12px 14px;
  }
  .grp {
    margin-top: 12px;
  }
  .grp:first-child {
    margin-top: 6px;
  }
  .glabel {
    font-size: 0.78rem;
    color: var(--faint);
    padding: 2px 12px 3px;
    font-weight: 500;
  }
  .grp:first-child .glabel {
    display: none;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 6px 12px;
    border-radius: 999px;
    color: var(--muted);
    text-decoration: none;
    font-size: 0.93rem;
    line-height: 1.4;
    font-weight: 500;
  }
  .item:hover {
    background: var(--surface-2);
    color: var(--text);
  }
  .item.on {
    background: var(--ink-soft);
    color: var(--ink);
    font-weight: 620;
  }
  .badge {
    margin-inline-start: auto;
    background: var(--heirloom);
    color: var(--surface);
    border-radius: 999px;
    font-size: 0.72rem;
    padding: 0 7px;
    font-weight: 600;
  }
  .foot {
    padding: 10px 18px 16px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .about-link {
    border: 0;
    background: none;
    padding: 0;
    font: inherit;
    color: inherit;
    cursor: pointer;
    text-align: start;
  }
  .about-link:hover {
    color: var(--ink);
    text-decoration: underline;
  }
  .credit a {
    color: inherit;
  }
  .credit a:hover {
    color: var(--ink);
  }
  .topbar,
  .tabbar {
    display: none;
  }
  .main {
    min-width: 0;
  }
  /* "Back" above the page, on the same left edge as its content */
  .backbar {
    width: 100%;
    max-width: var(--page-w, 1240px);
    margin: 0 auto;
    padding: 18px 36px 0;
  }
  .main.with-back > :global(.page) {
    padding-top: 10px;
  }
  @media (max-width: 860px) {
    .backbar {
      padding: 10px 16px 0;
    }
    .main.with-back > :global(.page) {
      padding-top: 6px;
    }
  }
  .rail.open {
    transform: none;
    visibility: visible;
    transition-delay: 0s;
  }
  :global([dir='rtl']) .rail:not(.open) {
    transform: translateX(calc(100% + var(--dock)));
  }
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(12, 20, 28, 0.4);
    z-index: 55;
  }
  .pre {
    white-space: pre-line;
  }

  .toasts {
    position: fixed;
    inset-inline-end: 18px;
    bottom: 18px;
    z-index: 2000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: min(420px, calc(100vw - 36px));
  }
  .toast {
    background: var(--text);
    color: var(--surface);
    padding: 11px 18px;
    border-radius: 999px;
    font-weight: 500;
    box-shadow: var(--shadow-pop);
    font-size: 0.93rem;
  }
  .toast.error {
    background: var(--danger);
    color: #fff;
  }

  @media (max-width: 900px) {
    .dock {
      display: none;
    }
    .shell {
      padding-inline-start: 0;
      padding-top: var(--topbar);
      padding-bottom: calc(var(--tabbar) + env(safe-area-inset-bottom));
    }
    .rail {
      inset-inline-start: 0;
      transform: translateX(-100%);
      width: min(300px, 86vw);
    }
    :global([dir='rtl']) .rail:not(.open) {
      transform: translateX(100%);
    }
    .rail.open {
      transform: none !important;
    }
    .close-menu {
      display: inline-flex;
    }
    .qs {
      display: none;
    }
    .topbar {
      display: flex;
      align-items: center;
      gap: 6px;
      position: fixed;
      top: 0;
      inset-inline: 0;
      height: calc(var(--topbar) + env(safe-area-inset-top));
      padding: env(safe-area-inset-top) 10px 0 6px;
      background: color-mix(in srgb, var(--paper) 88%, transparent);
      backdrop-filter: saturate(1.4) blur(12px);
      -webkit-backdrop-filter: saturate(1.4) blur(12px);
      z-index: 50;
    }
    .tb-title {
      font-weight: 560;
      font-size: 1.05rem;
      color: var(--text);
      text-decoration: none;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      flex: 1;
      min-width: 0;
    }
    .tb-search {
      flex: 0 1 220px;
      min-width: 0;
    }
    .tabbar {
      display: flex;
      position: fixed;
      bottom: 0;
      inset-inline: 0;
      height: calc(var(--tabbar) + env(safe-area-inset-bottom));
      padding-bottom: env(safe-area-inset-bottom);
      background: color-mix(in srgb, var(--surface) 92%, transparent);
      backdrop-filter: saturate(1.4) blur(12px);
      -webkit-backdrop-filter: saturate(1.4) blur(12px);
      border-top: 1px solid var(--line);
      z-index: 50;
    }
    .tabbar a,
    .tabbar button {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 3px;
      font: inherit;
      font-size: 0.7rem;
      color: var(--muted);
      text-decoration: none;
      background: none;
      border: 0;
      cursor: pointer;
      min-width: 0;
    }
    .tabbar span {
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      padding: 0 2px;
    }
    .tabbar a.on {
      color: var(--ink);
      font-weight: 650;
    }
    .tabbar a.on :global(svg) {
      background: var(--ink-soft);
      border-radius: 999px;
      padding: 3px 14px;
      box-sizing: content-box;
    }
    .toasts {
      bottom: calc(var(--tabbar) + 14px + env(safe-area-inset-bottom));
      inset-inline: 14px;
      max-width: none;
    }
  }
  .print-credit {
    display: none;
  }
  @media print {
    .shell {
      padding: 0 !important;
    }
    /* on every printed page, quietly */
    .print-credit {
      display: block;
      position: fixed;
      bottom: 3mm;
      right: 5mm;
      font-size: 7pt;
      color: #9a9a9a;
      letter-spacing: 0.02em;
    }
    /* the poster carries its own */
    :global(body:has(.poster-page)) .print-credit {
      display: none;
    }
  }
</style>
