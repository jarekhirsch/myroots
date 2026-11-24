<script lang="ts">
  // A photo at full size. People marked on it are listed under the photo and
  // outlined on hover; in "Mark people" mode faces are found on the device and
  // any face (or an area drawn with the mouse or a finger) can be named.
  import { ui, toast } from '../ui.svelte';
  import { app, call, summaries } from '../api.svelte';
  import { mediaSrc } from '../media';
  import { detectFaces, describeFaces } from '../faces';
  import { guessPhotoDate, guessToDate, type DateGuess } from '../photodate';
  import { fmtDate } from '../format';
  import DateInput from './DateInput.svelte';
  import CalendarClock from '../icons/CalendarDots.svelte';
  import Sparkles from '../icons/Sparkle.svelte';
  import { errorText } from '../errors';
  import { t, tn, tx } from '../i18n/index.svelte';
  import Img from './Img.svelte';
  import PersonPicker from './PersonPicker.svelte';
  import X from '../icons/X.svelte';
  import Left from '../icons/CaretLeft.svelte';
  import Right from '../icons/CaretRight.svelte';
  import ScanFace from '../icons/UserFocus.svelte';
  import Check from '../icons/Check.svelte';
  import Trash from '../icons/Trash.svelte';
  import type { MediaView, PhotoTag, TagView } from '../types';

  type Rect = [number, number, number, number];

  const lb = $derived(ui.lightbox!);
  const item = $derived(lb.items[lb.index]);
  const viewer = $derived(!!app.info?.viewer);

  let view = $state<MediaView | null>(null);
  let tagging = $state(false);
  let faces = $state<Rect[]>([]);
  let finding = $state(false);
  let hot = $state<number | null>(null);
  let edit = $state<null | { index: number | null; rect: Rect }>(null);
  let draft = $state<Rect | null>(null);
  let freeName = $state('');
  let stage: HTMLDivElement | undefined = $state();
  // face descriptors of the found faces (same order as `faces`) and who they may be
  let descs = $state<(number[] | null)[]>([]);
  let recognising = $state(false);
  let names = $state<Map<number, string>>(new Map());
  type Known = { person: number; media: number; face: number[] };
  let known: Known[] | null = null;

  const tags = $derived(view?.tags || []);
  const isImage = $derived(!!view && view.media.mime.startsWith('image/'));

  function close() {
    ui.lightbox = null;
  }
  function step(d: number) {
    if (!ui.lightbox) return;
    const n = ui.lightbox.items.length;
    ui.lightbox.index = (ui.lightbox.index + d + n) % n;
  }

  // the photo's marks
  $effect(() => {
    const id = item.media;
    view = null;
    faces = [];
    descs = [];
    dating = false;
    guess = null;
    edit = null;
    draft = null;
    hot = null;
    call<MediaView>('media.get', { id })
      .then((v) => {
        if (id === item.media) view = v;
      })
      .catch(() => {});
  });

  // faces, when marking
  $effect(() => {
    if (tagging && isImage) findFaces(item.media);
  });
  async function findFaces(id: number) {
    finding = true;
    faces = [];
    descs = [];
    let src = '';
    try {
      src = await mediaSrc(id, false);
      const f = await detectFaces(src);
      if (id === item.media) faces = f;
    } catch {
      /* no faces: areas can still be drawn */
    } finally {
      if (id === item.media) finding = false;
    }
    if (id !== item.media || !src) return;
    // who could it be: compare with the faces of people marked on other photos
    recognising = true;
    try {
      const [d, k, s] = await Promise.all([faces.length ? describeFaces(src, faces) : Promise.resolve([]), call<Known[]>('media.faces'), summaries()]);
      if (id !== item.media) return;
      descs = d;
      known = k;
      names = new Map(s.map((x) => [x.id, x.name]));
      // marks made before recognition existed get their descriptor now
      const missing = tags.filter((x) => x.person && !x.has_face);
      if (missing.length) {
        const md = await describeFaces(src, missing.map((x) => x.rect));
        const list = plain().map((x) => {
          const j = missing.findIndex((m) => m.rect === x.rect || (m.rect[0] === x.rect[0] && m.rect[1] === x.rect[1]));
          return j >= 0 && md[j] ? { ...x, face: md[j]! } : x;
        });
        if (md.some(Boolean) && id === item.media) await commit(list, true);
      }
    } catch {
      /* suggestions are optional */
    } finally {
      if (id === item.media) recognising = false;
    }
  }

  function distance(a: number[], b: number[]): number {
    let s = 0;
    for (let i = 0; i < a.length; i++) s += (a[i] - b[i]) * (a[i] - b[i]);
    return Math.sqrt(s);
  }
  // best match for each found face (people already marked on this photo are left out)
  const guesses = $derived.by(() => {
    const here = new Set(tags.map((x) => x.person).filter(Boolean));
    return faces.map((_, i) => {
      const d = descs[i];
      if (!d || !known?.length) return null;
      let best: { person: number; d: number } | null = null;
      for (const k of known) {
        if (here.has(k.person) || k.media === item.media) continue;
        const dd = distance(d, k.face);
        if (!best || dd < best.d) best = { person: k.person, d: dd };
      }
      return best && best.d < 0.55 ? { ...best, name: names.get(best.person) || '?' } : null;
    });
  });


  // intersection over union of two boxes
  function iou(a: Rect, b: Rect): number {
    const x = Math.max(0, Math.min(a[0] + a[2], b[0] + b[2]) - Math.max(a[0], b[0]));
    const y = Math.max(0, Math.min(a[1] + a[3], b[1] + b[3]) - Math.max(a[1], b[1]));
    const i = x * y;
    return i / Math.max(1e-9, a[2] * a[3] + b[2] * b[3] - i);
  }
  // found faces nobody is marked on yet
  const free = $derived(faces.filter((f) => !tags.some((tg) => iou(f, tg.rect) > 0.4)));
  const suggested = $derived(free.map((f) => guesses[faces.indexOf(f)]).filter(Boolean).length);

  const pct = (r: Rect) => `left:${r[0] * 100}%;top:${r[1] * 100}%;width:${r[2] * 100}%;height:${r[3] * 100}%`;

  /* ---------- drawing an area ---------- */
  let start: [number, number] | null = null;
  let moved = false;
  const clamp = (v: number, lo = 0, hi = 1) => Math.min(hi, Math.max(lo, v));
  function pos(e: PointerEvent): [number, number] {
    const r = stage!.getBoundingClientRect();
    return [clamp((e.clientX - r.left) / r.width), clamp((e.clientY - r.top) / r.height)];
  }
  function down(e: PointerEvent) {
    if (!tagging || !isImage || e.button !== 0) return;
    if ((e.target as HTMLElement).closest('.box, .face')) return;
    start = pos(e);
    moved = false;
    draft = null;
    stage!.setPointerCapture(e.pointerId);
    e.preventDefault();
  }
  function move(e: PointerEvent) {
    if (!start) return;
    const p = pos(e);
    const r: Rect = [Math.min(start[0], p[0]), Math.min(start[1], p[1]), Math.abs(p[0] - start[0]), Math.abs(p[1] - start[1])];
    if (r[2] > 0.012 || r[3] > 0.012) moved = true;
    if (moved) draft = r;
  }
  function up() {
    if (!start) return;
    const s = start;
    start = null;
    let r = draft;
    draft = null;
    if (!moved || !r || r[2] < 0.015 || r[3] < 0.015) {
      // a tap or a click: a square around that point
      const b = stage!.getBoundingClientRect();
      const side = 0.14 * Math.min(b.width, b.height);
      const w = side / b.width;
      const h = side / b.height;
      r = [clamp(s[0] - w / 2, 0, 1 - w), clamp(s[1] - h / 2, 0, 1 - h), w, h];
    }
    open(null, r);
  }

  /* ---------- naming ---------- */
  function open(index: number | null, rect: Rect) {
    const tg = index != null ? tags[index] : null;
    freeName = tg && !tg.person ? tg.name : '';
    edit = { index, rect };
  }

  function plain(): (PhotoTag & { face?: number[] })[] {
    return tags.map((x) => ({ person: x.person, name: x.name, rect: x.rect }));
  }
  async function commit(list: PhotoTag[], quiet = false) {
    const id = item.media;
    try {
      const v = await call<MediaView>('media.tags', { id, tags: list });
      if (id === item.media) view = v;
    } catch (e) {
      if (!quiet) toast(errorText(e), 'error');
    }
  }
  // the face descriptor of a box: from the found faces, or computed now
  async function faceOf(rect: Rect): Promise<number[] | undefined> {
    const i = faces.indexOf(rect);
    if (i >= 0 && descs[i]) return descs[i]!;
    try {
      return (await describeFaces(await mediaSrc(item.media, false), [rect]))[0] || undefined;
    } catch {
      return undefined;
    }
  }
  async function put(tag: PhotoTag & { face?: number[] }) {
    if (!edit) return;
    const at = edit.index;
    edit = null;
    if (tag.person != null) tag.face = await faceOf(tag.rect);
    let list = plain();
    if (at != null) list[at] = tag;
    else list.push(tag);
    // a person is marked once: the new mark replaces an older one
    if (tag.person != null) {
      const keep = at != null ? at : list.length - 1;
      list = list.filter((x, i) => i === keep || x.person !== tag.person);
    }
    commit(list);
  }
  // accept every suggestion at once
  function acceptAll() {
    let list = plain();
    for (const f of free) {
      const g = guesses[faces.indexOf(f)];
      if (!g || list.some((x) => x.person === g.person)) continue;
      list.push({ person: g.person, name: '', rect: f, face: descs[faces.indexOf(f)] || undefined });
    }
    commit(list);
  }

  /* ---------- when was it taken ---------- */
  let dating = $state(false);
  let dateDraft = $state('');
  let guessing = $state(false);
  let guess = $state<DateGuess | null>(null);
  const dated = $derived(tags.filter((x: TagView) => x.person && /\d{4}/.test(x.birth_date)));
  function openDating() {
    edit = null;
    tagging = false;
    dateDraft = view?.media.date || '';
    guess = null;
    dating = !dating;
  }
  async function estimate() {
    guessing = true;
    try {
      guess = await guessPhotoDate(await mediaSrc(item.media, false), dated);
      if (guess) dateDraft = guessToDate(guess);
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      guessing = false;
    }
  }
  async function saveDate() {
    const id = item.media;
    try {
      await call('media.update', { id, date: dateDraft });
      if (view && id === item.media) view = { ...view, media: { ...view.media, date: dateDraft } };
      dating = false;
      toast(t('Saved.'));
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }
  function remove() {
    if (!edit || edit.index == null) return;
    const at = edit.index;
    edit = null;
    commit(plain().filter((_, i) => i !== at));
  }

  function toggleTagging() {
    tagging = !tagging;
    edit = null;
    draft = null;
  }

  $effect(() => {
    const onKey = (e: KeyboardEvent) => {
      const typing = (e.target as HTMLElement)?.closest?.('input, textarea, select');
      if (e.key === 'Escape') {
        if (dating) dating = false;
        else if (edit) edit = null;
        else if (tagging) tagging = false;
        else close();
      } else if (typing || edit || dating) return;
      else if (e.key === 'ArrowRight') step(document.dir === 'rtl' ? -1 : 1);
      else if (e.key === 'ArrowLeft') step(document.dir === 'rtl' ? 1 : -1);
    };
    window.addEventListener('keydown', onKey);
    document.body.style.overflow = 'hidden';
    return () => {
      window.removeEventListener('keydown', onKey);
      document.body.style.overflow = '';
    };
  });

  let x0 = 0;
  const hint = $derived(
    finding
      ? t('Looking for faces…')
      : recognising
        ? t('Recognising faces…')
        : free.length
        ? tn('%d face without a name: click it to say who it is, or draw an area.', '%d faces without a name: click one to say who it is, or draw an area.', free.length)
        : t('Click a face or draw an area around it to mark a person.'),
  );
</script>

<div
  class="lb"
  class:tagging
  class:editing={!!edit || dating}
  role="dialog"
  tabindex="-1"
  aria-modal="true"
  aria-label={t('Gallery')}
  ontouchstart={(e) => (x0 = e.touches[0].clientX)}
  ontouchend={(e) => {
    if (tagging) return;
    const dx = e.changedTouches[0].clientX - x0;
    if (Math.abs(dx) > 50) step(dx < 0 ? 1 : -1);
  }}
>
  <button class="ctl x" onclick={close} aria-label={t('Close')}><X size={24} /></button>
  {#if !viewer && isImage}
    <div class="tools">
      <button class="tool" class:on={tagging} onclick={toggleTagging} aria-pressed={tagging}>
        {#if tagging}<Check size={17} /> {tx('finish marking people on a photo', 'Done')}{:else}<ScanFace size={17} /> {t('Mark people')}{/if}
      </button>
      {#if tagging && suggested && !recognising}
        <button class="tool" onclick={acceptAll}><Sparkles size={16} /> {tn('Accept %d suggestion', 'Accept %d suggestions', suggested)}</button>
      {/if}
      {#if !tagging}
        <button class="tool" class:on={dating} onclick={openDating} aria-pressed={dating}><CalendarClock size={17} /> {t('When was it taken?')}</button>
      {/if}
      {#if tagging}<span class="hint" aria-live="polite">{hint}</span>{/if}
    </div>
  {/if}
  {#if lb.items.length > 1 && !tagging}
    <button class="ctl nav prev" onclick={() => step(-1)} aria-label={t('Previous')}><Left size={30} /></button>
    <button class="ctl nav next" onclick={() => step(1)} aria-label={t('Next')}><Right size={30} /></button>
  {/if}
  <figure role="presentation" onclick={(e) => e.target === e.currentTarget && !tagging && close()}>
    <div class="stage" bind:this={stage} role="presentation" onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={() => ((start = null), (draft = null))}>
      {#key item.media}
        <Img id={item.media} thumb={false} alt={item.caption} />
      {/key}
      {#each tags as tg, i (i)}
        {#if tagging}
          <button type="button" class="box" class:hot={hot === i} class:cur={edit?.index === i} style={pct(tg.rect)} onclick={() => open(i, tg.rect)} aria-label={t('Change: %s', tg.label)}>
            <span class="nm">{tg.label}</span>
          </button>
        {:else if tg.person}
          <a class="box" class:hot={hot === i} style={pct(tg.rect)} href="#/person/{tg.person}" onclick={close} aria-label={tg.label} onmouseenter={() => (hot = i)} onmouseleave={() => (hot = null)}>
            <span class="nm">{tg.label}</span>
          </a>
        {:else}
          <span class="box" class:hot={hot === i} style={pct(tg.rect)} role="img" aria-label={tg.label} onmouseenter={() => (hot = i)} onmouseleave={() => (hot = null)}>
            <span class="nm">{tg.label}</span>
          </span>
        {/if}
      {/each}
      {#if tagging}
        {#each free as f, i (i)}
          {@const g = guesses[faces.indexOf(f)]}
          <button type="button" class="face" class:sug={!!g} class:cur={!!edit && edit.index == null && edit.rect === f} style={pct(f)} onclick={() => open(null, f)} aria-label={g ? t('Is this %s?', g.name) : t('Who is this?')}>
            <span class="q">?</span>
            {#if g}<span class="nm">{g.name}?</span>{/if}
          </button>
        {/each}
        {#if draft}<span class="draft" style={pct(draft)}></span>{/if}
        {#if edit && edit.index == null && !free.includes(edit.rect)}<span class="draft" style={pct(edit.rect)}></span>{/if}
      {/if}
    </div>
    {#if item.caption || lb.items.length > 1 || tags.length || view?.media.date}
      <figcaption>
        {#if item.caption}<span class="cap">{item.caption}</span>{/if}
        {#if view?.media.date}<span class="n">{fmtDate(view.media.date)}</span>{/if}
        {#if lb.items.length > 1}<span class="n">{lb.index + 1} / {lb.items.length}</span>{/if}
        {#if tags.length}
          <span class="who">
            {t('In the photo:')}
            {#each tags as tg, i (i)}
              {#if tg.person && !tagging}<a href="#/person/{tg.person}" onclick={close} onmouseenter={() => (hot = i)} onmouseleave={() => (hot = null)} onfocus={() => (hot = i)} onblur={() => (hot = null)}>{tg.label}</a>{:else}<span class="pn" role="presentation" onmouseenter={() => (hot = i)} onmouseleave={() => (hot = null)}>{tg.label}</span>{/if}{i < tags.length - 1 ? ', ' : ''}
            {/each}
          </span>
        {/if}
      </figcaption>
    {/if}
  </figure>

  {#if edit}
    {@const cur = edit.index != null ? tags[edit.index] : null}
    <div class="pop" role="dialog" aria-label={t('Who is this?')}>
      <div class="ph">
        <b>{cur ? cur.label : t('Who is this?')}</b>
        <button type="button" class="btn ghost icon sm" onclick={() => (edit = null)} aria-label={t('Cancel')}><X size={16} /></button>
      </div>
      {#if edit.index == null}
        {@const g = guesses[faces.indexOf(edit.rect)]}
        {#if g}
          <div class="sugg">
            <span>{t('Is this %s?', g.name)}</span>
            <button type="button" class="btn sm primary" onclick={() => put({ person: g.person, name: '', rect: edit!.rect })}>{t('Yes')}</button>
          </div>
        {/if}
      {/if}
      <span class="lbl">{t('A person in the tree')}</span>
      <PersonPicker value={cur?.person ?? null} create={{}} placeholder={t('Search or type a new name…')} onchange={(id) => id && put({ person: id, name: '', rect: edit!.rect })} />
      <form class="free" onsubmit={(e) => (e.preventDefault(), freeName.trim() && put({ person: null, name: freeName.trim(), rect: edit!.rect }))}>
        <label class="lbl" for="lb-free">{t('Or only a name, without adding to the tree')}</label>
        <div class="row">
          <input id="lb-free" type="text" bind:value={freeName} placeholder={t('e.g. Aunt Ala, a neighbour')} />
          <button class="btn sm" type="submit" disabled={!freeName.trim()}>{t('Save')}</button>
        </div>
      </form>
      {#if cur}
        <button type="button" class="btn ghost sm danger rm" onclick={remove}><Trash size={14} /> {t('Remove the mark')}</button>
      {/if}
    </div>
  {/if}

  {#if dating}
    <div class="pop" role="dialog" aria-label={t('When was it taken?')}>
      <div class="ph">
        <b>{t('When was it taken?')}</b>
        <button type="button" class="btn ghost icon sm" onclick={() => (dating = false)} aria-label={t('Cancel')}><X size={16} /></button>
      </div>
      <DateInput bind:value={dateDraft} />
      {#if dated.length}
        <button type="button" class="btn sm" onclick={estimate} disabled={guessing}><Sparkles size={14} /> {guessing ? t('Estimating…') : t('Estimate from the faces')}</button>
        {#if guess}
          <div class="guess small">
            {#if guess.best != null}
              <p>{t('Probably about %1$d (between %2$d and %3$d).', guess.best, guess.from, guess.to)}</p>
            {:else}
              <p>{t('Taken between %1$d and %2$d: everyone marked was already born and still alive.', guess.from, guess.to)}</p>
            {/if}
            <ul>
              {#each guess.basis as b (b.name)}
                <li>{b.age != null ? t('%1$s: looks about %2$d years old (born %3$d)', b.name, b.age, b.born) : t('%1$s: born %2$d', b.name, b.born)}</li>
              {/each}
            </ul>
          </div>
        {/if}
      {:else}
        <p class="small muted">{t('Mark people with a known year of birth on the photo, and the app can estimate when it was taken.')}</p>
      {/if}
      <div class="row end">
        <button type="button" class="btn sm primary" onclick={saveDate}>{t('Save')}</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .lb {
    position: fixed;
    inset: 0;
    z-index: 3000;
    background: rgba(8, 12, 16, 0.94);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  figure {
    margin: 0;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 64px 24px;
    gap: 12px;
  }
  .stage {
    position: relative;
    display: inline-block;
    max-width: 100%;
    line-height: 0;
  }
  .tagging .stage {
    cursor: crosshair;
    touch-action: none;
    user-select: none;
    -webkit-user-select: none;
  }
  .stage :global(img) {
    display: block;
    max-width: 100%;
    max-height: calc(100vh - 190px);
    max-height: calc(100dvh - 190px);
    object-fit: contain;
    border-radius: 4px;
    -webkit-user-drag: none;
  }
  .stage :global(.ph) {
    width: 200px;
    height: 200px;
    background: rgba(255, 255, 255, 0.06);
  }

  /* marks */
  .box,
  .face,
  .draft {
    position: absolute;
    box-sizing: border-box;
    border-radius: 6px;
    line-height: normal;
  }
  .box {
    border: 2px solid rgba(255, 255, 255, 0.92);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.35), inset 0 0 0 1px rgba(0, 0, 0, 0.25);
    opacity: 0;
    transition: opacity 0.15s;
    background: none;
    padding: 0;
    color: inherit;
    text-decoration: none;
  }
  .stage:hover .box,
  .box.hot,
  .box:focus-visible,
  .tagging .box {
    opacity: 1;
  }
  .stage:hover .box:not(.hot) {
    opacity: 0.55;
  }
  .stage:hover .box:hover {
    opacity: 1;
  }
  .tagging .box {
    cursor: pointer;
    border-color: var(--heirloom, #d6b36a);
  }
  .nm {
    position: absolute;
    top: calc(100% + 5px);
    left: 50%;
    transform: translateX(-50%);
    white-space: nowrap;
    background: #fff;
    color: #172026;
    font-size: 0.78rem;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 999px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
    pointer-events: none;
    opacity: 0;
  }
  .box.hot .nm,
  .box:hover .nm,
  .box:focus-visible .nm,
  .tagging .box .nm {
    opacity: 1;
  }
  .face {
    border: 2px dashed rgba(255, 255, 255, 0.85);
    background: rgba(255, 255, 255, 0.06);
    cursor: pointer;
    padding: 0;
  }
  .face:hover,
  .face:focus-visible {
    background: rgba(255, 255, 255, 0.16);
    border-style: solid;
  }
  .q {
    position: absolute;
    top: -11px;
    inset-inline-end: -11px;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: #fff;
    color: #172026;
    font-weight: 700;
    font-size: 0.8rem;
    display: grid;
    place-items: center;
  }
  .draft {
    border: 2px solid var(--heirloom, #d6b36a);
    background: color-mix(in srgb, var(--heirloom, #d6b36a) 18%, transparent);
    pointer-events: none;
  }
  .cur {
    border-color: var(--heirloom, #d6b36a) !important;
    border-style: solid !important;
    background: color-mix(in srgb, var(--heirloom, #d6b36a) 18%, transparent) !important;
    opacity: 1 !important;
  }

  figcaption {
    color: #e8edf2;
    font-size: 0.95rem;
    text-align: center;
    max-width: 80ch;
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 2px 10px;
  }
  .n {
    color: #9aa7b3;
    font-size: 0.85rem;
  }
  .who {
    flex-basis: 100%;
    color: #9aa7b3;
    font-size: 0.88rem;
  }
  .who a,
  .who .pn {
    color: #e8edf2;
  }
  .who a {
    text-decoration: underline;
    text-decoration-color: rgba(232, 237, 242, 0.35);
    text-underline-offset: 3px;
  }
  .who a:hover {
    text-decoration-color: currentColor;
  }

  /* controls */
  .ctl {
    position: absolute;
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    border: 0;
    border-radius: 50%;
    width: 46px;
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 2;
  }
  .ctl:hover {
    background: rgba(255, 255, 255, 0.18);
  }
  .x {
    top: 12px;
    inset-inline-end: 12px;
  }
  .prev {
    inset-inline-start: 12px;
  }
  .next {
    inset-inline-end: 12px;
  }
  :global([dir='rtl']) .nav :global(svg) {
    transform: scaleX(-1);
  }
  .tools {
    position: absolute;
    top: 14px;
    inset-inline-start: 14px;
    inset-inline-end: 72px;
    display: flex;
    align-items: center;
    gap: 14px;
    z-index: 2;
  }
  .tool {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 40px;
    padding: 0 16px;
    border-radius: 999px;
    border: 0;
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
    font: inherit;
    font-size: 0.92rem;
    font-weight: 600;
    cursor: pointer;
    flex: none;
  }
  .tool:hover {
    background: rgba(255, 255, 255, 0.2);
  }
  .tool.on {
    background: #fff;
    color: #172026;
  }
  .tool:focus-visible,
  .ctl:focus-visible,
  .box:focus-visible,
  .face:focus-visible {
    outline: 2px solid #fff;
    outline-offset: 2px;
  }
  .hint {
    color: #c9d2da;
    font-size: 0.88rem;
  }

  /* naming panel */
  .pop {
    position: absolute;
    top: 68px;
    inset-inline-end: 16px;
    width: 340px;
    max-width: calc(100% - 32px);
    background: var(--surface, #fff);
    color: var(--ink, #172026);
    border-radius: 16px;
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.45);
    padding: 14px 16px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 4;
  }
  .ph {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .ph b {
    font-size: 1.02rem;
  }
  .lbl {
    font-size: 0.8rem;
    color: var(--muted);
  }
  .free {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 6px;
    padding-top: 10px;
    border-top: 1px solid var(--line, rgba(0, 0, 0, 0.1));
  }
  .free .row {
    display: flex;
    gap: 6px;
  }
  .free input {
    flex: 1;
    min-width: 0;
  }
  .rm {
    align-self: flex-start;
  }
  .sugg {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--heirloom, #d6b36a) 18%, transparent);
    font-weight: 600;
  }
  .face.sug {
    border-color: var(--heirloom, #d6b36a);
  }
  .face .nm {
    opacity: 1;
    background: var(--heirloom, #d6b36a);
  }
  .guess p {
    margin: 4px 0;
  }
  .guess ul {
    margin: 0;
    padding-inline-start: 18px;
    color: var(--muted);
  }
  .row.end {
    display: flex;
    justify-content: flex-end;
  }
  @media (min-width: 900px) {
    .editing figure {
      padding-inline-end: 380px;
    }
  }
  @media (max-width: 640px) {
    figure {
      padding: 64px 8px 16px;
    }
    .nav {
      top: auto;
      bottom: 14px;
    }
    .tools {
      flex-wrap: wrap;
      gap: 6px 10px;
    }
    .hint {
      font-size: 0.8rem;
    }
    /* at the top: the on-screen keyboard and the list of people need the space below */
    .pop {
      top: 8px;
      inset-inline: 8px;
      width: auto;
      max-width: none;
    }
  }
</style>
