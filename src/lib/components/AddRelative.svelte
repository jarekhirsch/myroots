<script lang="ts">
  // Adds a relative to a person: a brand-new person or a link to someone already in the tree.
  import Modal from './Modal.svelte';
  import PersonPicker from './PersonPicker.svelte';
  import DateInput from './DateInput.svelte';
  import { call } from '../api.svelte';
  import { t } from '../i18n/index.svelte';
  import { toast } from '../ui.svelte';
  import { errorText } from '../errors';
  import { go } from '../router.svelte';
  import { masculineSurname, feminineSurname } from '../text';
  import type { Profile } from '../types';

  type Rel = 'father' | 'mother' | 'spouse' | 'child' | 'sibling';
  let { profile, relation, onclose }: { profile: Profile; relation: Rel; onclose: () => void } = $props();

  let mode = $state<'new' | 'existing'>('new');
  let first = $state('');
  let last = $state('');
  let gender = $state(relation === 'father' ? 'M' : relation === 'mother' ? 'F' : relation === 'spouse' ? (profile.person.gender === 'M' ? 'F' : profile.person.gender === 'F' ? 'M' : '') : '');
  let birth = $state('');
  let death = $state('');
  let other = $state<number | null>(null);
  let existing = $state<number | null>(null);
  let busy = $state(false);
  let openAfter = $state(false);

  // Surname suggestion: the family name in the right grammatical gender
  // (Kowalska → Kowalski). It follows the gender and the other parent until
  // the surname is typed by hand.
  let lastTouched = $state(false);
  const suggested = $derived.by(() => {
    const p = profile.person;
    const own = p.gender === 'F' ? p.maiden_name || p.last_name : p.last_name;
    const gendered = (s: string) => (!s ? '' : gender === 'F' ? feminineSurname(s) : gender === 'M' ? masculineSurname(s) : s);
    if (relation === 'father') return masculineSurname(own);
    if (relation === 'sibling') return gendered(own);
    if (relation === 'child') {
      if (p.gender === 'F') {
        const dad = partners.find((x) => x.id === other && x.gender === 'M');
        return gendered(dad ? dad.last_name : p.last_name);
      }
      return gendered(p.last_name);
    }
    return '';
  });
  $effect(() => {
    if (!lastTouched) last = suggested;
  });

  const partners = $derived(profile.unions.filter((u) => u.partner).map((u) => u.partner!));
  $effect(() => {
    if (relation === 'child' && partners.length === 1 && other === null) other = partners[0].id;
  });

  const titles: Record<Rel, () => string> = {
    father: () => t('Add father'),
    mother: () => t('Add mother'),
    spouse: () => t('Add spouse or partner'),
    child: () => t('Add child'),
    sibling: () => t('Add sibling'),
  };

  async function save() {
    busy = true;
    try {
      let id: number | null = null;
      if (mode === 'existing') {
        if (!existing) return;
        await call('person.link', { of: profile.person.id, other: existing, relation });
        id = existing;
      } else {
        if (!first.trim() && !last.trim()) return;
        const r = await call<{ id: number }>('person.add_relative', {
          of: profile.person.id,
          relation,
          person: { first_name: first.trim(), last_name: last.trim(), gender, birth_date: birth, death_date: death },
          other_parent: relation === 'child' ? other : null,
        });
        id = r.id;
      }
      toast(t('Saved.'));
      onclose();
      if (openAfter && id) go(`/person/${id}/edit`);
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }

  const exclude = $derived([profile.person.id]);
</script>

<Modal title={titles[relation]()} {onclose}>
  <div class="seg" role="tablist">
    <button role="tab" aria-selected={mode === 'new'} onclick={() => (mode = 'new')}>{t('New person')}</button>
    <button role="tab" aria-selected={mode === 'existing'} onclick={() => (mode = 'existing')}>{t('Person already in the tree')}</button>
  </div>

  <form id="addrel" onsubmit={(e) => { e.preventDefault(); save(); }}>
    {#if mode === 'existing'}
      <label class="field">
        <span class="lbl">{t('Person')}</span>
        <PersonPicker bind:value={existing} {exclude} gender={relation === 'father' ? 'M' : relation === 'mother' ? 'F' : ''} />
      </label>
      {#if relation === 'sibling'}
        <p class="hint">{t('The person will get the same parents as %s.', profile.summary.name)}</p>
      {/if}
    {:else}
      <div class="form-grid">
        <label class="field"><span class="lbl">{t('First name')}</span><input type="text" bind:value={first} /></label>
        <label class="field"><span class="lbl">{t('Surname')}</span><input type="text" bind:value={last} oninput={() => (lastTouched = true)} /></label>
        <label class="field">
          <span class="lbl">{t('Gender')}</span>
          <select bind:value={gender}>
            <option value="">—</option>
            <option value="M">{t('Male')}</option>
            <option value="F">{t('Female')}</option>
            <option value="U">{t('Unknown')}</option>
          </select>
        </label>
        <div></div>
        <label class="field"><span class="lbl">{t('Date of birth')}</span><DateInput bind:value={birth} /></label>
        <label class="field"><span class="lbl">{t('Date of death')}</span><DateInput bind:value={death} /></label>
        {#if relation === 'child' && partners.length}
          <label class="field full">
            <span class="lbl">{t('Other parent')}</span>
            <select bind:value={other}>
              <option value={null}>{t('Unknown')}</option>
              {#each partners as p (p.id)}<option value={p.id}>{p.name}</option>{/each}
            </select>
          </label>
        {/if}
      </div>
      <label class="check after"><input type="checkbox" bind:checked={openAfter} /> {t('Open the full form after saving')}</label>
    {/if}
  </form>

  {#snippet footer()}
    <button class="btn" onclick={onclose}>{t('Cancel')}</button>
    <button class="btn primary" type="submit" form="addrel" disabled={busy || (mode === 'existing' ? !existing : !first.trim() && !last.trim())}>{t('Save')}</button>
  {/snippet}
</Modal>

<style>
  .seg {
    display: flex;
    gap: 4px;
    background: var(--surface-2);
    box-shadow: var(--shadow);
    border-radius: 14px;
    padding: 3px;
    margin-bottom: 16px;
  }
  .seg button {
    flex: 1;
    border: 0;
    background: none;
    font: inherit;
    font-size: 0.9rem;
    padding: 7px 8px;
    border-radius: 7px;
    cursor: pointer;
    color: var(--muted);
  }
  .seg button[aria-selected='true'] {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
    font-weight: 550;
  }
  .after {
    margin-top: 14px;
  }
</style>
