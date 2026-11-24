// Navigation of the app and of the read-only family website.
import { t, tx } from './i18n/index.svelte';
import Home from './icons/House.svelte';
import Users from './icons/Users.svelte';
import Tree from './icons/TreeStructure.svelte';
import MapIcon from './icons/MapTrifold.svelte';
import Images from './icons/Images.svelte';
import Paw from './icons/PawPrint.svelte';
import Scroll from './icons/Scroll.svelte';
import Calendar from './icons/CalendarHeart.svelte';
import Chart from './icons/ChartBar.svelte';
import Link from './icons/LinkSimple.svelte';
import Book from './icons/BookOpen.svelte';
import Check from './icons/ShieldCheck.svelte';
import Merge from './icons/GitMerge.svelte';
import Inbox from './icons/Tray.svelte';
import Flag from './icons/Flag.svelte';
import Pin from './icons/MapPin.svelte';
import Cog from './icons/GearSix.svelte';
import Database from './icons/Database.svelte';
import Globe from './icons/GlobeSimple.svelte';
import Flower from './icons/FlowerTulip.svelte';
import Frame from './icons/FrameCorners.svelte';
import FolderTree from './icons/Folders.svelte';

export interface NavItem {
  path: string;
  label: () => string;
  /** Shorter label for the dock and the phone tab bar. */
  short?: () => string;
  icon: any;
  /** Section key in app.info.sections (family website privacy). */
  section?: string;
  editorOnly?: boolean;
  badge?: 'suggestions';
}

export interface NavGroup {
  label: () => string;
  items: NavItem[];
}

export const NAV: NavGroup[] = [
  {
    label: () => t('Family'),
    items: [
      { path: '/', label: () => t('Start'), icon: Home },
      { path: '/people', label: () => t('People'), icon: Users, section: 'list' },
      { path: '/tree', label: () => t('Family tree'), short: () => tx('nav', 'Tree'), icon: Tree, section: 'tree' },
      { path: '/map', label: () => t('Map'), icon: MapIcon, section: 'map' },
      { path: '/gallery', label: () => t('Gallery'), icon: Images, section: 'gallery' },
      { path: '/events', label: () => t('Events'), icon: Calendar, section: 'list' },
      { path: '/graves', label: () => t('Graves'), icon: Flower, section: 'list' },
      { path: '/pets', label: () => t('Pets'), icon: Paw, section: 'list' },
      { path: '/sources', label: () => t('Sources'), icon: Scroll, editorOnly: true },
    ],
  },
  {
    label: () => t('Explore'),
    items: [
      { path: '/stats', label: () => t('Statistics'), icon: Chart, section: 'stats' },
      { path: '/kinship', label: () => t('Kinship'), icon: Link, section: 'kinship' },
      { path: '/book', label: () => t('Genealogy book'), icon: Book, section: 'list' },
      { path: '/poster', label: () => t('Poster'), icon: Frame, section: 'tree' },
    ],
  },
  {
    label: () => t('Tidy up'),
    items: [
      { path: '/health', label: () => t('Data consistency'), icon: Check, editorOnly: true },
      { path: '/duplicates', label: () => t('Possible duplicates'), icon: Merge, editorOnly: true },
      { path: '/suggestions', label: () => t('Suggestions'), icon: Inbox, editorOnly: true, badge: 'suggestions' },
      { path: '/clans', label: () => t('Clans'), icon: Flag, editorOnly: true },
      { path: '/places', label: () => t('Places'), icon: Pin, editorOnly: true },
    ],
  },
  {
    label: () => t('Settings'),
    items: [
      { path: '/website', label: () => t('Family website'), icon: Globe, editorOnly: true },
      { path: '/trees', label: () => t('Family trees'), icon: FolderTree, editorOnly: true },
      { path: '/data', label: () => t('Import and backup'), icon: Database, editorOnly: true },
      { path: '/settings', label: () => t('Settings'), icon: Cog, editorOnly: true },
    ],
  },
];

/** Items of the bottom bar on phones (the rest is under "More"). */
export const TABBAR = ['/', '/people', '/tree', '/map'];

/** Items of the narrow dock on wide screens (the rest is in the menu panel). */
export const DOCK = ['/', '/people', '/tree', '/map', '/gallery', '/events'];

export function visible(item: NavItem, viewer: boolean, sections: Record<string, boolean> | undefined): boolean {
  if (viewer && item.editorOnly) return false;
  if (viewer && item.section && sections && sections[item.section] === false) return false;
  if (item.path === '/map' && viewer && sections && sections.map === false) return false;
  return true;
}

export function isActive(path: string, current: string): boolean {
  if (path === '/') return current === '/';
  const first = '/' + (current.split('/')[1] || '');
  const map: Record<string, string> = { '/person': '/people', '/pet': '/pets', '/source': '/sources', '/lifeline': '/people' };
  return first === path || map[first] === path;
}
