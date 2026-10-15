<script lang="ts">
  import { call } from '../lib/api.svelte';
  import { go, back, route } from '../lib/router.svelte';
  import { t } from '../lib/i18n/index.svelte';
  import { speciesLabel, BREEDS } from '../lib/format';
  import { toast } from '../lib/ui.svelte';
  import { errorText } from '../lib/errors';
  import PersonPicker from '../lib/components/PersonPicker.svelte';
  import DateInput from '../lib/components/DateInput.svelte';
  import RichText from '../lib/components/RichText.svelte';
  import PhotoField from '../lib/components/PhotoField.svelte';
  import GalleryEditor from '../lib/components/GalleryEditor.svelte';
  import type { Pet, PetView } from '../lib/types';

  let { id }: { id: number | null } = $props();

  function blank(): Pet {
    return { id: 0, status: 'published', name: '', species: 'dog', breed: '', gender: '', birth_date: '', death_date: '', owner: route.query.owner ? Number(route.query.owner) : null, biography: '', notes: '', photo: null, gallery: [], created: 0, updated: 0 };
  }
  let pet = $state<Pet>(blank());
  let loaded = $state(false);
  let busy = $state(false);
  const listId = 'breeds';

  $effect(() => {
    const cur = id;
    loaded = false;
    if (cur) {
      call<PetView>('pet.get', { id: cur }).then((v) => {
        pet = { ...blank(), ...v.pet };
        loaded = true;
      });
    } else {
      pet = blank();
      loaded = true;
    }
  });

  async function save() {
    if (!pet.name.trim()) {
      toast(t('Enter a name.'), 'error');
      return;
    }
    busy = true;
    try {
      const r = await call<{ id: number }>('pet.save', { pet: { ...pet, id: id || 0 } });
      toast(t('Saved.'));
      go(`/pet/${r.id}`, true);
    } catch (e) {
      toast(errorText(e), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<div class="page">
  <div class="page-head"><h1>{id ? t('Edit pet') : t('Add pet')}</h1></div>
  {#if loaded}
    <form onsubmit={(e) => { e.preventDefault(); save(); }}>
      <div class="panel">
        <div class="form-grid">
          <label class="field"><span class="lbl">{t('Name')}</span><input type="text" bind:value={pet.name} required /></label>
          <label class="field">
            <span class="lbl">{t('Species')}</span>
            <select bind:value={pet.species}>
              {#each ['dog', 'cat', 'bird', 'horse', 'other'] as s}<option value={s}>{speciesLabel(s)}</option>{/each}
              {#if pet.species && !['dog', 'cat', 'bird', 'horse', 'other'].includes(pet.species)}<option value={pet.species}>{pet.species}</option>{/if}
            </select>
          </label>
          <label class="field">
            <span class="lbl">{t('Breed')}</span>
            <input type="text" bind:value={pet.breed} list={listId} />
            <datalist id={listId}>{#each BREEDS[pet.species] || [] as b}<option value={t(b)}></option>{/each}</datalist>
          </label>
          <label class="field">
            <span class="lbl">{t('Gender')}</span>
            <select bind:value={pet.gender}>
              <option value="">—</option>
              <option value="F">{t('Female')}</option>
              <option value="M">{t('Male')}</option>
            </select>
          </label>
          <label class="field"><span class="lbl">{t('Date of birth')}</span><DateInput bind:value={pet.birth_date} /></label>
          <label class="field"><span class="lbl">{t('Date of death')}</span><DateInput bind:value={pet.death_date} /></label>
          <div class="field full"><span class="lbl">{t('Owner')}</span><PersonPicker bind:value={pet.owner} /></div>
          <label class="field">
            <span class="lbl">{t('Status')}</span>
            <select bind:value={pet.status}>
              <option value="published">{t('Published')}</option>
              <option value="draft">{t('Draft')}</option>
              <option value="private">{t('Private')}</option>
            </select>
          </label>
        </div>
      </div>
      <div class="panel"><span class="lbl-s">{t('Photo')}</span><PhotoField bind:value={pet.photo} round /></div>
      <div class="panel"><span class="lbl-s">{t('Biography')}</span><RichText bind:value={pet.biography} label={t('Biography')} /></div>
      <div class="panel"><h2>{t('Gallery')}</h2><GalleryEditor bind:items={pet.gallery} /></div>
      <div class="panel"><span class="lbl-s">{t('Notes')}</span><RichText bind:value={pet.notes} label={t('Notes')} minHeight={90} /></div>
      <div class="savebar">
        <button type="button" class="btn" onclick={() => back(id ? `/pet/${id}` : '/pets')}>{t('Cancel')}</button>
        <button class="btn primary" type="submit" disabled={busy}>{t('Save')}</button>
      </div>
    </form>
  {/if}
</div>

<style>
  .panel {
    margin-bottom: 14px;
  }
  .panel h2 {
    font-size: 1rem;
  }
  .lbl-s {
    display: block;
    font-size: 0.86rem;
    color: var(--muted);
    font-weight: 500;
    margin-bottom: 8px;
  }
  .savebar {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    position: sticky;
    bottom: 0;
    padding: 12px 0;
    background: linear-gradient(transparent, var(--paper) 30%);
  }
  @media (max-width: 900px) {
    .savebar {
      bottom: calc(var(--tabbar) + env(safe-area-inset-bottom));
    }
  }
</style>
