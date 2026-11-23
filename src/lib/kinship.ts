// Kinship terms, ported from the WordPress plugin (MyRoots_Kinship) so that
// its translations of these phrases work unchanged.
import { t, tx, tn } from './i18n/index.svelte';
import type { KinshipResult, Gender } from './types';

function ancestorTerm(k: number, g: Gender): string {
  if (k === 1) return g === 'F' ? t('mother') : g === 'M' ? t('father') : t('parent');
  const base = g === 'F' ? t('grandmother') : g === 'M' ? t('grandfather') : t('grandparent');
  return tx('prefix for great-grandparents', 'great-').repeat(Math.max(0, k - 2)) + base;
}

function ancestorTermGenitive(k: number): string {
  if (k === 1) return tx('genitive', 'parent');
  const pra = tx('prefix for great-grandparents', 'great-').repeat(Math.max(0, k - 2));
  return t('%1$s or %2$s', pra + tx('genitive', 'grandfather'), pra + tx('genitive', 'grandmother'));
}

export function hopTerm(edge: string, g: Gender): string {
  if (edge === 'parent') return g === 'F' ? t('mother') : g === 'M' ? t('father') : t('parent');
  if (edge === 'child') return g === 'F' ? t('daughter') : g === 'M' ? t('son') : t('child');
  if (edge === 'spouse') return g === 'F' ? t('wife') : g === 'M' ? t('husband') : t('spouse');
  return '';
}

/** [relation sentence, detail sentence] */
export function kinshipText(k: KinshipResult): [string, string] {
  const nameA = k.name_a;
  const nameB = k.name_b;
  const gA = k.gender_a;
  const gB = k.gender_b;
  if (k.type === 'same') return [t('This is the same person.'), ''];
  if (k.type === 'none') return [t('No connection'), t('No family path was found between these people.')];
  if (k.type === 'affinity') {
    let term = '';
    switch (k.signature) {
      case 'spouse':
        term = gB === 'F' ? t('wife') : gB === 'M' ? t('husband') : t('spouse');
        break;
      case 'spouse,parent':
        term = gB === 'F' ? t('mother-in-law') : t('father-in-law');
        break;
      case 'parent,spouse':
        term = gB === 'F' ? t('stepmother') : t('stepfather');
        break;
      case 'spouse,child':
        term = gB === 'F' ? t('stepdaughter') : t('stepson');
        break;
      case 'child,spouse':
        term = gB === 'F' ? t('daughter-in-law') : t('son-in-law');
        break;
      case 'spouse,parent,child':
      case 'parent,child,spouse':
        term = gB === 'F' ? t('sister-in-law') : t('brother-in-law');
        break;
    }
    const rel = term ? t('%1$s is the %2$s of %3$s', nameB, term, nameA) : t('%1$s and %2$s are related by marriage.', nameA, nameB);
    return [rel, t('No common ancestor: the people are connected by marriages.')];
  }
  // blood
  const dA = k.depth_a;
  const dB = k.depth_b;
  if (dB === 0) return [t('%1$s is the %2$s of %3$s', nameB, ancestorTerm(dA, gB), nameA), tn('%d generation apart.', '%d generations apart.', dA)];
  if (dA === 0) return [t('%1$s is the %2$s of %3$s', nameA, ancestorTerm(dB, gA), nameB), tn('%d generation apart.', '%d generations apart.', dB)];
  if (dA === 1 && dB === 1) {
    let term = gA === 'F' ? t('sister') : gA === 'M' ? t('brother') : t('sibling');
    if (!k.full) term = (gA === 'F' ? tx('feminine prefix', 'half-') : tx('masculine prefix', 'half-')) + term;
    return [t('%1$s is the %2$s of %3$s', nameA, term, nameB), k.full ? t('Both parents in common.') : t('One parent in common.')];
  }
  if (dA === 1 || dB === 1) {
    const uIsA = dA === 1;
    const uName = uIsA ? nameA : nameB;
    const nName = uIsA ? nameB : nameA;
    const uG = uIsA ? gA : gB;
    const deep = uIsA ? dB : dA;
    let term: string;
    if (deep === 2) term = uG === 'F' ? t('aunt') : uG === 'M' ? t('uncle') : t('aunt/uncle');
    else {
      const sib = uG === 'F' ? t('sister') : uG === 'M' ? t('brother') : t('sibling');
      term = tx('kinship: sibling of an ancestor', '%1$s of a %2$s', sib, ancestorTermGenitive(deep - 1));
    }
    return [t('%1$s is the %2$s of %3$s', uName, term, nName), t('Sibling of an ancestor; generation difference: %d.', deep - 1)];
  }
  const degree = Math.min(dA, dB) - 1;
  const removal = Math.abs(dA - dB);
  const term = gA === 'F' ? tx('female', 'cousin') : tx('male', 'cousin');
  const rel = t('%1$s is a %3$d. degree %2$s of %4$s', nameA, term, degree, nameB);
  return [rel, removal ? t('Additionally removed by %d generation(s).', removal) : t('Same generation.')];
}
