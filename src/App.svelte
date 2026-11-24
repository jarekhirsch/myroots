<script lang="ts">
  import { assetUrl, wpBack } from './lib/backend/assets';
  import WpIcon from './lib/icons/ArrowSquareUpLeft.svelte';
  import { app, call, refreshInfo } from './lib/api.svelte';
  import UpdateBanner from './lib/components/UpdateBanner.svelte';
  import ProgressBar from './lib/components/ProgressBar.svelte';
  import { checkUpdates } from './lib/updates.svelte';
  import { route, go, back, canGoBack } from './lib/router.svelte';
  import { ui, toast } from './lib/ui.svelte';
  import { errorText, isStorageFull } from './lib/errors';
  import { t, lang } from './lib/i18n/index.svelte';
  import { NAV, TABBAR, DOCK, visible, isActive } from './lib/nav';
  import { backend } from './lib/backend';
  import Modal from './lib/components/Modal.svelte';
  import Lightbox from './lib/components/Lightbox.svelte';
  import QuickSearch from './lib/components/QuickSearch.svelte';
  import SearchPalette from './lib/components/SearchPalette.svelte';
  import SearchIcon from './lib/icons/MagnifyingGlass.svelte';
  import BackButton from './lib/components/BackButton.svelte';
  import AboutDialog from './lib/components/AboutDialog.svelte';
  import ReportDialog from './lib/components/ReportDialog.svelte';
  import Bug from './lib/icons/Bug.svelte';
  import Heart from './lib/icons/Heart.svelte';
  import { supportUrl } from './lib/credits';
  import { AUTHOR, YEAR, authorUrl } from './lib/credits';
  import MenuIcon from './lib/icons/List.svelte';
  import More from './lib/icons/DotsThree.svelte';
  import X from './lib/icons/X.svelte';

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
  import TreesView from './views/TreesView.svelte';
  import { trees, syncTreeName, hasTrees } from './lib/trees.svelte';
  import FolderTree from './lib/icons/Folders.svelte';
  import LockIcon from './lib/icons/LockKey.svelte';
  import LogOut from './lib/icons/SignOut.svelte';
  import { isServerHost, wpConfig } from './lib/backend/assets';
  import Unlock from './lib/components/Unlock.svelte';
  import Website from './views/Website.svelte';
  import NotFound from './views/NotFound.svelte';

  const viewer = $derived(!!app.info?.viewer);
  const sections = $derived(app.info?.sections);
  // A new, empty tree shows only what helps to start.
  const START_ONLY = ['/', '/data', '/settings', '/trees'];
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

  // Quick search: Ctrl K / Cmd K from every screen, or "Search" at the top of the menu.
  let palette = $state(false);
  const canSearch = $derived(!app.info?.empty && (!viewer || sections?.list !== false));
  $effect(() => {
    const k = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === 'k' && canSearch) {
        e.preventDefault();
        ui.menuOpen = false;
        palette = !palette;
      }
    };
    window.addEventListener('keydown', k);
    return () => window.removeEventListener('keydown', k);
  });
  $effect(() => {
    void route.path;
    palette = false;
  });
  const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

  $effect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && ui.menuOpen) ui.menuOpen = false;
    };
    window.addEventListener('keydown', k);
    return () => window.removeEventListener('keydown', k);
  });

  // Desktop app: a new version from myroots.cc (at most once a day, can be turned off)
  let updChecked = false;
  $effect(() => {
    if (viewer || updChecked || !app.settings) return;
    updChecked = true;
    setTimeout(() => checkUpdates().catch(() => {}), 8000);
  });

  // Several trees: keep the name in the list of trees and in the settings in step.
  let treesSynced = false;
  $effect(() => {
    if (viewer || treesSynced || !app.info || !app.settings || !hasTrees()) return;
    treesSynced = true;
    syncTreeName().catch(() => {});
  });

  // Family website with protected people: the way to type the password.
  const siteLock = $derived(viewer ? ((backend as any).lock && !(backend as any).unlocked ? (backend as any).lock : null) : null);
  let unlockOpen = $state(false);
  $effect(() => {
    const open = () => (unlockOpen = true);
    window.addEventListener('myroots-unlock', open);
    return () => window.removeEventListener('myroots-unlock', open);
  });

  let about = $state(false);
  let report = $state(false);
  // Inside WordPress: the way back to the site and the save state of the editor.
  const wpLink = wpBack();
  const serverHost = isServerHost();
  const wpCfg = wpConfig();
  let wpSave = $state({ saving: false, error: '' });
  $effect(() => {
    const b = backend as any;
    if (b.platform !== 'wp' || !b.listeners) return;
    const l = (s: { saving: boolean; error: string }) => (wpSave = s);
    b.listeners.add(l);
    return () => b.listeners.delete(l);
  });
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
  const editorOnly = $derived(['sources', 'source', 'health', 'duplicates', 'suggestions', 'clans', 'places', 'settings', 'data', 'website', 'trees'].includes(p[0] || ''));

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

  // A change could not be written to the browser storage (usually: no room left).
  // Shown at most every 30 s, so a series of edits does not flood the screen.
  $effect(() => {
    let last = 0;
    const h = (e: Event) => {
      if (Date.now() - last < 30_000) return;
      last = Date.now();
      const err = (e as CustomEvent).detail;
      toast(isStorageFull(err) ? errorText(err) : t('Changes could not be saved on this device. Make a backup now.'), 'error');
    };
    window.addEventListener('myroots-save-failed', h);
    return () => window.removeEventListener('myroots-save-failed', h);
  });

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
      <a href="#/" class="dock-logo" aria-label={title}><img src={assetUrl('icon.svg')} alt="" width="40" height="40" /></a>
      {#if canSearch}
        <button class="dk search" onclick={() => (palette = true)} aria-keyshortcuts={isMac ? 'Meta+K' : 'Control+K'}>
          <span class="dk-ic"><SearchIcon size={22} /></span>
          <span class="dk-l">{t('Search')}<kbd>{isMac ? '⌘K' : 'Ctrl K'}</kbd></span>
        </button>
      {/if}
      {#each dock as item (item!.path)}
        {@const Icon = item!.icon}
        <a href="#{item!.path}" class="dk" class:on={isActive(item!.path, route.path)} aria-current={isActive(item!.path, route.path) ? 'page' : undefined}>
          <span class="dk-ic"><Icon size={21} /></span>
          <span class="dk-l">{(item!.short || item!.label)()}</span>
        </a>
      {/each}
      {#if !viewer}
        <a class="dk support" href={supportUrl(lang())} target="_blank" rel="noopener" title={t('Support the project')}>
          <span class="dk-ic"><Heart size={18} /></span>
          <span class="dk-l">{t('Support')}</span>
        </a>
      {/if}
      <button class="dk more" class:on={inMenu || ui.menuOpen} onclick={() => (ui.menuOpen = !ui.menuOpen)} aria-expanded={ui.menuOpen}>
        <span class="dk-ic"><MenuIcon size={21} />{#if newSuggestions && !viewer}<span class="dot" aria-hidden="true"></span>{/if}</span>
        <span class="dk-l">{t('More')}</span>
      </button>
    </nav>
    <aside class="rail no-print" class:open={ui.menuOpen} aria-label={t('Menu')}>
      <div class="brand">
        <a href="#/" class="logo" aria-label="MyRoots">
          <img src={assetUrl('icon.svg')} alt="" width="36" height="36" />
          <span class="nm">
            <span class="name">{title}</span>
            {#if viewer && app.info?.site?.subtitle}<span class="sub">{app.info.site.subtitle}</span>{:else if !viewer && title !== 'MyRoots'}<span class="sub">MyRoots</span>{/if}
          </span>
        </a>
        <button class="btn ghost icon close-menu" onclick={() => (ui.menuOpen = false)} aria-label={t('Close')}><X size={18} /></button>
      </div>
      {#if !viewer && trees.list.length > 1}
        <a href="#/trees" class="tree-switch" onclick={() => (ui.menuOpen = false)}><FolderTree size={15} /><span>{t('Switch tree (%d)', trees.list.length)}</span></a>
      {/if}
      {#if !app.info?.empty && (!viewer || sections?.list !== false)}
        <div class="qs"><QuickSearch /></div>
      {/if}
      <nav>
        <!-- "More": only what is not already in the dock (computer) or in the tab bar (phone) -->
        {#each groups as g (g.label())}
          <div class="grp" class:alldock={g.items.every((i) => DOCK.includes(i.path))} class:alltab={g.items.every((i) => TABBAR.includes(i.path))}>
            <div class="glabel">{g.label()}</div>
            {#each g.items as item (item.path)}
              {@const Icon = item.icon}
              <a href="#{item.path}" class="item" class:indock={DOCK.includes(item.path)} class:intab={TABBAR.includes(item.path)} class:on={isActive(item.path, route.path)} aria-current={isActive(item.path, route.path) ? 'page' : undefined}>
                <Icon size={18} />
                <span>{item.label()}</span>
                {#if item.badge === 'suggestions' && newSuggestions}<span class="badge">{newSuggestions}</span>{/if}
              </a>
            {/each}
          </div>
        {/each}
      </nav>
      {#if siteLock}
        <div class="unlock-link">
          <button type="button" class="item" onclick={() => ((unlockOpen = true), (ui.menuOpen = false))}><LockIcon size={18} /><span>{t('Show protected people')}</span></button>
        </div>
      {/if}
      {#if wpLink}
        <div class="wp-back">
          <a href={wpLink.url} class="item"><WpIcon size={18} /><span>{viewer ? wpLink.label || t('Back to the website') : serverHost ? wpLink.label : t('Back to WordPress')}</span></a>
          {#if serverHost && wpCfg?.logout}<a href={wpCfg.logout} class="item"><LogOut size={18} /><span>{t('Log out')}</span></a>{/if}
          {#if !viewer}<p class="wp-state small" class:err={!!wpSave.error} role="status">{wpSave.error ? t('Not saved yet, retrying…') : wpSave.saving ? t('Saving…') : serverHost ? t('Saved on the server') : t('Saved in WordPress')}</p>{/if}
        </div>
      {/if}
      {#if !viewer}
        <div class="report-link">
          <button type="button" class="item" onclick={() => ((report = true), (ui.menuOpen = false))}><Bug size={18} /><span>{t('Report a problem')}</span></button>
          <!-- discreet: a quiet line, never in the family website -->
          <a class="item support" href={supportUrl(lang())} target="_blank" rel="noopener" onclick={() => (ui.menuOpen = false)}><Heart size={18} /><span>{t('Support the project')}</span></a>
        </div>
      {/if}
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
      {#if canSearch}<button class="btn soft icon tb-search" onclick={() => (palette = true)} aria-label={t('Search')}><SearchIcon size={20} /></button>{/if}
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
      {:else if p[0] === 'trees'}
        <TreesView />
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
          <Icon size={24} />
          <span>{(item!.short || item!.label)()}</span>
        </a>
      {/each}
      <button onclick={() => (ui.menuOpen = true)}>
        <More size={24} />
        <span>{t('More')}</span>
      </button>
    </nav>
  </div>

  {#if palette}<SearchPalette onclose={() => (palette = false)} />{/if}
  {#if !viewer}<UpdateBanner /><ProgressBar />{/if}
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
{#if report}<ReportDialog onclose={() => (report = false)} />{/if}
{#if unlockOpen && siteLock}<Modal title={t('Protected people')} onclose={() => (unlockOpen = false)}><Unlock header={siteLock} /></Modal>{/if}
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
    background: var(--surface-2);
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
  .dk.search {
    margin-bottom: 10px;
  }
  .dk.search .dk-ic {
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .dk.search kbd {
    display: block;
    font: inherit;
    font-size: 0.66rem;
    color: var(--faint);
    margin-top: 1px;
  }
  .dk.support {
    margin-top: auto;
    color: var(--muted);
  }
  .dk.support .dk-l {
    font-weight: 450;
  }
  .dk.more {
    margin-top: auto;
  }
  .dk.support + .dk.more {
    margin-top: 2px;
  }
  .dot {
    position: absolute;
    top: 3px;
    inset-inline-end: 8px;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--heirloom);
    box-shadow: 0 0 0 2px var(--surface-2);
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
  .tree-switch {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: -6px 16px 8px 20px;
    font-size: 0.84rem;
    color: var(--muted);
    text-decoration: none;
  }
  .tree-switch:hover {
    color: var(--ink);
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
  .unlock-link,
  .report-link {
    padding: 8px 10px 4px;
    border-top: 1px solid var(--line);
  }
  .report-link .support {
    color: var(--muted);
    text-decoration: none;
  }
  .unlock-link .item,
  .report-link .item {
    width: 100%;
    border: 0;
    background: none;
    font: inherit;
    font-size: 0.93rem;
    cursor: pointer;
    color: var(--ink);
  }
  .wp-back {
    padding: 0 10px 4px;
    border-top: 1px solid var(--line);
    padding-top: 8px;
  }
  .wp-state {
    margin: 2px 12px 0;
    color: var(--faint);
  }
  .wp-state.err {
    color: var(--danger, #b3261e);
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

  /* computer: the dock already has these */
  @media (min-width: 901px) {
    .rail .item.indock,
    .rail .grp.alldock {
      display: none;
    }
  }
  @media (max-width: 900px) {
    .rail .item.intab,
    .rail .grp.alltab {
      display: none;
    }
    .dock {
      display: none;
    }
    .shell {
      padding-inline-start: 0;
      padding-top: var(--topbar);
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
      flex: none;
      border-radius: 999px;
    }
    /* floating pill in the accent color; the active place is a white circle */
    .tabbar {
      display: flex;
      gap: 6px;
      position: fixed;
      left: 50%;
      bottom: calc(22px + env(safe-area-inset-bottom));
      transform: translateX(-50%);
      padding: 8px 10px;
      border-radius: 999px;
      background: var(--ink);
      box-shadow: 0 10px 30px rgba(18, 22, 19, 0.2);
      z-index: 50;
      animation: navup 0.5s cubic-bezier(0.2, 0.8, 0.2, 1) 0.15s backwards;
    }
    :global([dir='rtl']) .tabbar {
      transform: translateX(50%);
      left: auto;
      right: 50%;
    }
    .tabbar a,
    .tabbar button {
      width: 48px;
      height: 48px;
      flex: none;
      display: grid;
      place-items: center;
      border-radius: 999px;
      color: color-mix(in srgb, var(--surface) 86%, transparent);
      background: none;
      border: 0;
      cursor: pointer;
      transition: background 0.2s, color 0.2s;
    }
    .tabbar a.on {
      background: var(--surface);
      color: var(--ink);
    }
    .tabbar span {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0 0 0 0);
    }
    @keyframes navup {
      from {
        opacity: 0;
        translate: 0 30px;
      }
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
