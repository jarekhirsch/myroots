// Godparents, witnesses and other people tied to a person (GEDCOM ASSO).
import { t } from './i18n/index.svelte';

/** Roles known to the app; anything else is free text written by the user or a file. */
export const ROLES = ['godparent', 'witness', 'guardian', 'clergy', 'officiator', 'friend', 'neighbor'] as const;

/** Label of a role. `reverse`: seen from the other side ("Godchild" for a godparent). */
export function roleLabel(role: string, reverse = false): string {
  const direct: Record<string, () => string> = {
    godparent: () => t('Godparent'),
    witness: () => t('Witness'),
    guardian: () => t('Guardian'),
    clergy: () => t('Clergy'),
    officiator: () => t('Officiator'),
    friend: () => t('Friend'),
    neighbor: () => t('Neighbor'),
  };
  const back: Record<string, () => string> = {
    godparent: () => t('Godchild'),
    witness: () => t('Was a witness for'),
    guardian: () => t('Ward'),
    clergy: () => t('Clergy for'),
    officiator: () => t('Officiated for'),
    friend: () => t('Friend'),
    neighbor: () => t('Neighbor'),
  };
  const m = (reverse ? back : direct)[role];
  if (m) return m();
  if (!role) return reverse ? t('Related to') : t('Related person');
  return reverse ? t('%s for', role) : role;
}
