<script lang="ts">
  // "Through the years" of one person: the animation, and (in the editor)
  // choosing the photos, their dates and the position of the eyes.
  import { app, call } from '../lib/api.svelte';
  import { useLoad } from '../lib/load.svelte';
  import { t, tn } from '../lib/i18n/index.svelte';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import { pickImages } from '../lib/media';
  import { detectEyes, warmUp } from '../lib/faces';
  import { photoUrl } from '../lib/aging';
  import { year as yearOf } from '../lib/format';
  import AgingPlayer from '../lib/components/AgingPlayer.svelte';
  import EyeAligner from '../lib/components/EyeAligner.svelte';
  import DateInput from '../lib/components/DateInput.svelte';
  import Img from '../lib/components/Img.svelte';
  import FramePreview from '../lib/components/FramePreview.svelte';
  import Modal from '../lib/components/Modal.svelte';
  import NotFound from './NotFound.svelte';
  import Upload from '../lib/icons/UploadSimple.svelte';
  import Images from '../lib/icons/Images.svelte';
  import X from '../lib/icons/X.svelte';
  import Crosshair from '../lib/icons/Crosshair.svelte';
  import Check from '../lib/icons/Check.svelte';
  import type { Profile, AgingFrame } from '../lib/types';

  let { id }: { id: number } = $props();
  const viewer = $derived(!!app.info?.viewer);
  const prof = useLoad(() => call<Profile>('person.profile', { id }));
  const P = $derived(prof.data);
  const p = $derived(P?.person);

  let frames = $state<AgingFrame[]>([]);
  let audio = $state<number | null>(null);
  let loadedFor = 0;
  $effect(() => {
    if (p && loadedFor !== p.id) {
      loadedFor = p.id;
      frames = (p.aging || []).map((f) => ({ ...f }));
      audio = p.aging_audio ?? null;
    }
  });
  $effect(() => {
    if (!viewer) warmUp();
  });

  /* ---------- saving (a moment after each change) ---------- */
  let saveTimer = 0;
  let lastSaved = '';
  $effect(() => {
    const sig = JSON.stringify({ frames, audio });
    if (!loadedFor || viewer) return;
    if (!lastSaved) {
      lastSaved = sig;
      return;
    }
    if (sig === lastSaved) return;
    clearTimeout(saveTimer);
    saveTimer = window.setTimeout(() => save(sig), 500);
  });
  async function save(sig: string, addToGallery: number[] = []) {
    try {
      const r = await call<{ aging: AgingFrame[]; aging_audio: number | null }>('person.set_media', { id, aging: $state.snapshot(frames), aging_audio: audio, add_to_gallery: addToGallery });
      lastSaved = sig;
      // the engine keeps them in date order
      if (r.aging.map((f) => f.media).join() !== frames.map((f) => f.media).join()) {
        frames = r.aging;
        lastSaved = JSON.stringify({ frames, audio });
      }
    } catch (e) {
      toast(errorText(e), 'error');
    }
  }

  /* ---------- adding photos ---------- */
  let choosing = $state(false);
  let picked = $state<number[]>([]);
  let detecting = $state<{ done: number; total: number } | null>(null);
  const candidates = $derived.by(() => {
    if (!p) return [] as { media: number; caption: string }[];
    const have = new Set(frames.map((f) => f.media));
    const all = [...(p.photo ? [{ media: p.photo, caption: '' }] : []), ...p.gallery];
    const seen = new Set<number>();
    return all.filter((g) => !have.has(g.media) && !seen.has(g.media) && seen.add(g.media));
  });

  function guessDate(caption: string): string {
    const m = caption.match(/\b(1[89]\d\d|20\d\d)\b/);
    return m ? m[1] : '';
  }

  async function addFrames(ids: number[], captions: Map<number, string> = new Map(), toGallery = false) {
    if (!ids.length) return;
    const fresh = ids.filter((m) => !frames.some((f) => f.media === m)).map((media) => ({ media, date: guessDate(captions.get(media) || ''), eyes: null }) as AgingFrame);
    frames = [...frames, ...fresh];
    if (toGallery) await save(JSON.stringify({ frames, audio }), ids);
    detecting = { done: 0, total: fresh.length };
    for (const f of fresh) {
      try {
        const eyes = await detectEyes(await photoUrl(f.media));
        const cur = frames.find((x) => x.media === f.media);
        if (cur && eyes && !cur.eyes) cur.eyes = eyes;
      } catch (e) {
        console.warn('face detection', e);
      }
      detecting = { done: detecting!.done + 1, total: fresh.length };
    }
    detecting = null;
    const missing = fresh.filter((f) => !frames.find((x) => x.media === f.media)?.eyes).length;
    if (missing) toast(tn('On %d photo the face was not found: set the eyes by hand.', 'On %d photos the face was not found: set the eyes by hand.', missing));
  }

  async function upload() {
    const ids = await pickImages(true);
    await addFrames(ids, new Map(), true);
  }
  function addPicked() {
    const caps = new Map(candidates.map((c) => [c.media, c.caption]));
    const ids = picked.slice();
    choosing = false;
    picked = [];
    addFrames(ids, caps);
  }

  /* ---------- editing ---------- */
  let aligning = $state<AgingFrame | null>(null);
  function setEyes(eyes: [number, number, number, number]) {
    const f = frames.find((x) => x.media === aligning?.media);
    if (f) f.eyes = eyes;
    aligning = null;
  }
  function remove(m: number) {
    frames = frames.filter((f) => f.media !== m);
  }
  const birthYear = $derived(yearOf(p?.birth_date || ''));
  function ageAt(f: AgingFrame): string {
    const y = yearOf(f.date);
    if (y == null || birthYear == null || y < birthYear) return '';
    return tn('%d year', '%d years', y - birthYear);
  }
  const ordered = $derived(
    frames
      .map((f, i) => ({ f, i }))
      .sort((a, b) => (a.f.date && b.f.date ? a.f.date.replace(/^[~<>]/, '').localeCompare(b.f.date.replace(/^[~<>]/, '')) : a.f.date ? -1 : b.f.date ? 1 : a.i - b.i))
      .map((x) => x.f),
  );
</script>

{#if prof.error}
  <NotFound private={(prof.error as any)?.code === 'private'} />
{:else if P && p}
  <div class="page">
    <div class="page-head">
      <div>
        <h1>{t('Through the years')}</h1>
        <p class="lead"><a href="#/person/{id}">{P.summary.name}</a>{#if P.summary.years}&nbsp;<span class="muted">{P.summary.years}</span>{/if}</p>
      </div>
    </div>

    {#if viewer && ordered.length < 2}
      <div class="panel empty">{t('No photos yet.')}</div>
    {:else}
      <div class="grid" class:solo={viewer}>
        <div class="player">
          {#if ordered.length}
            <AgingPlayer frames={ordered} birth={p.birth_date} name={P.summary.name} audio={audio} />
          {:else}
            <div class="ph">
              <Images size={34} strokeWidth={1.5} />
              <p>{t('Add photos of this person from different periods of life: childhood, wedding, old age… The faces are lined up and the photos play one after another.')}</p>
            </div>
          {/if}
        </div>

        {#if !viewer}
          <div class="editor">
            <div class="ed-head">
              <h2>{tn('%d photo', '%d photos', frames.length)}</h2>
              <div class="actions">
                {#if candidates.length}<button class="btn sm" onclick={() => ((choosing = true), (picked = []))}><Images size={15} /> {t('From their photos')}</button>{/if}
                <button class="btn sm ghost" onclick={upload}><Upload size={15} /> {t('Upload photos')}</button>
              </div>
            </div>
            {#if detecting}
              <p class="notice">{t('Looking for faces… %1$d of %2$d', detecting.done, detecting.total)}</p>
            {/if}

            <ul class="frames">
              {#each ordered as f (f.media)}
                <li class="fr">
                  <button class="th" onclick={() => (aligning = f)} title={t('Set the eyes')} aria-label={t('Set the eyes')}>
                    <FramePreview frame={f} />
                  </button>
                  <div class="meta">
                    <div class="date"><DateInput bind:value={f.date} /></div>
                    <div class="st">
                      {#if ageAt(f)}<span class="age">{ageAt(f)}</span>{/if}
                      {#if f.eyes}
                        <span class="ok"><Check size={13} /> {t('Face lined up')}</span>
                      {:else}
                        <span class="warn">{t('Set the eyes')}</span>
                      {/if}
                      {#if !f.date}<span class="warn">{t('Add the year')}</span>{/if}
                    </div>
                  </div>
                  <div class="ctl">
                    <button class="btn ghost icon sm" onclick={() => (aligning = f)} title={t('Set the eyes')} aria-label={t('Set the eyes')}><Crosshair size={16} /></button>
                    <button class="btn ghost icon sm" onclick={() => remove(f.media)} title={t('Remove')} aria-label={t('Remove')}><X size={16} /></button>
                  </div>
                </li>
              {/each}
            </ul>

            {#if p.recordings?.length}
              <label class="field sound">
                <span class="lbl">{t('Sound')}</span>
                <select bind:value={audio}>
                  <option value={null}>{t('No sound')}</option>
                  {#each p.recordings as r (r.media)}<option value={r.media}>{r.title || t('Recording')}</option>{/each}
                </select>
              </label>
            {:else}
              <p class="small muted">{t('Tip: add a voice recording to the person (Edit → Photos) and it can play along with the photos.')}</p>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if choosing}
    <Modal title={t('Choose photos')} onclose={() => (choosing = false)} wide>
      <div class="pick">
        {#each candidates as c (c.media)}
          <button class="pc" class:on={picked.includes(c.media)} aria-pressed={picked.includes(c.media)} onclick={() => (picked = picked.includes(c.media) ? picked.filter((x) => x !== c.media) : [...picked, c.media])}>
            <Img id={c.media} alt={c.caption} />
            {#if picked.includes(c.media)}<span class="tick"><Check size={16} /></span>{/if}
          </button>
        {/each}
      </div>
      {#snippet footer()}
        <button class="btn ghost" onclick={() => (choosing = false)}>{t('Cancel')}</button>
        <button class="btn primary" onclick={addPicked} disabled={!picked.length}>{tn('Add %d photo', 'Add %d photos', picked.length)}</button>
      {/snippet}
    </Modal>
  {/if}

  {#if aligning}
    <EyeAligner frame={aligning} onsave={setEyes} onclose={() => (aligning = null)} />
  {/if}
{/if}

<style>
  .grid {
    display: grid;
    grid-template-columns: minmax(280px, 440px) minmax(0, 1fr);
    gap: 32px;
    align-items: start;
  }
  .grid.solo {
    grid-template-columns: minmax(0, 520px);
  }
  .player {
    position: sticky;
    top: 20px;
  }
  .ph {
    aspect-ratio: 4 / 5;
    border-radius: var(--radius-lg);
    border: 2px dashed var(--line);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 30px;
    text-align: center;
    color: var(--muted);
  }
  .ph p {
    margin: 0;
    max-width: 34ch;
  }
  .ed-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 12px;
  }
  .ed-head h2 {
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .frames {
    list-style: none;
    margin: 0 0 18px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .fr {
    display: grid;
    grid-template-columns: 76px minmax(0, 1fr) auto;
    gap: 14px;
    align-items: center;
    padding: 10px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow);
  }
  .th {
    position: relative;
    width: 76px;
    height: 95px;
    padding: 0;
    border: 0;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    background: var(--ink-soft);
  }
  .meta {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .date {
    max-width: 320px;
  }
  .st {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
    font-size: 0.82rem;
  }
  .age {
    color: var(--ink);
    font-weight: 600;
  }
  .ok {
    color: var(--muted);
    display: inline-flex;
    gap: 3px;
    align-items: center;
  }
  .warn {
    color: var(--warn, #9a6200);
    font-weight: 600;
  }
  .ctl {
    display: flex;
    gap: 2px;
  }
  .sound {
    max-width: 360px;
  }
  .pick {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 10px;
  }
  .pc {
    position: relative;
    aspect-ratio: 4 / 5;
    border-radius: 10px;
    overflow: hidden;
    border: 3px solid transparent;
    padding: 0;
    cursor: pointer;
    background: var(--ink-soft);
  }
  .pc.on {
    border-color: var(--ink);
  }
  .pc :global(img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .tick {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--ink);
    color: var(--surface);
  }
  @media (max-width: 860px) {
    .grid {
      grid-template-columns: minmax(0, 1fr);
    }
    .player {
      position: static;
      max-width: 460px;
    }
  }
</style>
