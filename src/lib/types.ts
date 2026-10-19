// TypeScript mirrors of the Rust model (crates/myroots-core/src/model.rs).
export type Id = number;
export type Gender = 'M' | 'F' | 'U' | '';
export type ParentRel = 'bio' | 'adopted' | 'foster' | 'step';
export type Status = 'published' | 'draft' | 'private';

export interface LifeEvent { name: string; date: string; place: string; description: string; image: Id | null }
export interface GalleryItem { media: Id; caption: string }
export interface Recording { media: Id; title: string; date: string }
/** eyes: left x, left y, right x, right y as fractions of the image (as seen on the photo). */
export interface AgingFrame { media: Id; date: string; eyes: [number, number, number, number] | null }

export interface Person {
  id: Id; status: Status;
  first_name: string; last_name: string; maiden_name: string; gender: Gender;
  birth_date: string; birth_place: string; death_date: string; death_place: string; death_cause: string; deceased?: boolean;
  biography: string; notes: string;
  occupation: string; education: string; faith: string; height: string; eye_color: string; blood_type: string;
  mother: Id | null; mother_rel: ParentRel; father: Id | null; father_rel: ParentRel;
  events: LifeEvent[]; clans: string[];
  photo: Id | null; gallery: GalleryItem[];
  recordings?: Recording[]; aging?: AgingFrame[]; aging_audio?: Id | null; grave?: Grave;
  custom: Record<string, Record<string, string>>;
  citations: Record<string, Id[]>;
  created: number; updated: number;
}

export interface Union {
  id: Id; a: Id; b: Id | null;
  partner_first_name: string; partner_last_name: string;
  date: string; place: string; end_cause: string; end_date: string; informal: boolean;
}

export interface Pet {
  id: Id; status: Status; name: string; species: string; breed: string; gender: Gender;
  birth_date: string; death_date: string; owner: Id | null; biography: string; notes: string;
  photo: Id | null; gallery: GalleryItem[]; created: number; updated: number;
}

export interface Source {
  id: Id; title: string; date: string; url: string; notes: string; media_url: string;
  photo: Id | null; scans: Id[]; persons: Id[]; created: number; updated: number;
}

export interface Grave { cemetery: string; location: string; lat: number | null; lng: number | null; photo: Id | null; url: string; notes: string }
export interface GraveEntry { person: Summary; grave: Grave; death_date: string }
export interface PhotoTag { person: Id | null; name: string; rect: [number, number, number, number]; face?: number[] }
export interface Media { id: Id; file_name: string; mime: string; size: number; width: number; height: number; has_thumb: boolean; caption: string; origin: string; created: number; tags?: PhotoTag[]; date?: string }
export interface TagView extends PhotoTag { label: string; birth_date: string; death_date: string; has_face: boolean }
export interface MediaView { media: Media; tags: TagView[] }
export interface DayWeather { code: number; tmax: number | null; tmin: number | null; rain: number | null; snow: number | null }

export interface Summary {
  id: Id; name: string; first_name: string; last_name: string; maiden_name: string; gender: Gender;
  birth_date: string; birth_place: string; death_date: string; death_place: string; years: string;
  clans: string[]; photo: Id | null; living: boolean; status: Status; occupation: string; updated: number;
  anon?: boolean; censored?: boolean;
}

export interface UnionView { union: Union; partner: Summary | null; partner_name: string; children: Id[] }
export interface SourceRef { id: Id; title: string; date: string; url: string; notes: string; media_url: string; photo: Id | null; scans: Id[]; fields: string[]; linked: boolean }
export interface PetRef { id: Id; name: string; species: string; breed: string; photo: Id | null; years: string }

export interface Profile {
  person: Person; summary: Summary; age: number | null; living: boolean; presumed_dead: boolean;
  father: Summary | null; mother: Summary | null; unions: UnionView[];
  children: Summary[]; siblings: Summary[]; half_siblings: Id[];
  pets: PetRef[]; sources: SourceRef[]; grandparents: Summary[]; grandchildren: Summary[];
  revisions: number; censored: boolean;
  tagged?: { media: Id; caption: string }[];
}

export interface EditModel { person: Person; unions: UnionView[]; children: Id[]; sources: Id[] }

export interface CustomField { id: string; name: string; type: string; options: string[] }
export interface CustomGroup { id: string; name: string; fields: CustomField[] }

export interface Settings {
  tree_name: string; max_age: number;
  privacy: { enabled: boolean; level: string; sections: string[]; fields: string[] };
  map_external: boolean;
  weather_off: boolean;
  appearance: { primary: string; accent: string; bg: string; text: string; border: string; font: string; font_size: string; radius: string; shadow: string; theme: string; map_style: string };
  custom_schema: CustomGroup[]; clans: string[]; sort_order: string; people_per_page: number;
  tree_start_mode: string; tree_start_person: Id | null; merge_dismissed: string[];
  notifications: { enabled: boolean; days_before: number };
  suggestions: { enabled: boolean; email: string; endpoint: string };
  site: { title: string; subtitle: string; intro: string; language: string; full_photos: boolean };
  language: string;
}

export interface AppInfo {
  version: string; viewer: boolean; data_version: number;
  sections: Record<string, boolean>; tree_name: string;
  site: Settings['site']; appearance: Settings['appearance']; language: string;
  suggestions: Settings['suggestions']; map_external: boolean; weather_external?: boolean; empty: boolean;
}

export interface Overview {
  tree_name: string; persons: number; unions: number; pets: number; sources: number; media: number;
  places: number; clans: number; living: number; new_suggestions: number; version: number; recent: Summary[]; empty: boolean;
}

export interface Anniversary { date: string; days_left: number; kind: 'birth' | 'death' | 'marriage'; years: number; living: boolean; person: Id; name: string; partner: Id | null; partner_name: string; photo: Id | null }
export interface Revision { id: Id; person: Id; time: number; fields: string[]; data: Person; unions: Union[] }
export interface ClanInfo { name: string; count: number; defined: boolean }
export interface PlaceUse { name: string; count: number; lat: number | null; lng: number | null; origin: string; place_id: Id | null; weather?: Record<string, DayWeather> }
export interface PetView { pet: Pet; owner: Summary | null; years: string; alive: boolean }
export interface SourceView { source: Source; persons: Summary[]; citations: [Summary, string[]][] }
export interface GalleryEntry { media: Id; caption: string; kind: string; owner_id: Id; owner_name: string; owner_type: 'person' | 'pet' | 'source'; clans: string[]; date: string }
export interface FamilyEvent { kind: string; name: string; date: string; place: string; description: string; image: Id | null; person: Id; person_name: string; photo: Id | null; clans: string[]; with_name: string; with_id: Id | null; cause: string }
export interface Suggestion { id: Id; person: Id | null; person_name: string; message: string; author: string; email: string; status: string; created: number; token: string }

export interface LEvent { kind: string; date: string; place: string; name: string; description: string; image: Id | null; related: Id | null; related_name: string; role: string; age: number | null }
export interface Lifeline { person: Id; name: string; gender: Gender; photo: Id | null; years: string; living?: boolean; events: LEvent[] }

export interface MapEvent { person: Id; name: string; photo: Id | null; clans: string[]; kind: string; label: string; date: string }
export interface MapPlace { place: string; lat: number | null; lng: number | null; events: MapEvent[] }
export interface MapData { geocoded: MapPlace[]; uncached: MapPlace[] }

export interface NamedAge { id: Id; name: string; age: number }
export interface Stats {
  total: number; males: number; females: number; living: number; pets_alive: number; pets_dead: number; pets_total: number;
  avg_age: number | null; oldest: NamedAge | null; youngest: NamedAge | null; marriages: number; avg_children: number | null;
  avg_marriage_age_m: number | null; avg_marriage_age_f: number | null; avg_gen_gap: number | null;
  names_top10: [string, number][]; surnames_top10: [string, number][]; birth_places_top10: [string, number][];
  birth_decades: [string, number][]; death_decades: [string, number][]; age_groups: [string, number][]; children_dist: [string, number][];
  months_of_birth: number[]; occupations_top10: [string, number][]; generations: number;
}

export interface PathNode { id: Id; name: string; years: string; gender: Gender; photo: Id | null; edge: string }
export interface KinshipResult { type: 'same' | 'blood' | 'affinity' | 'none'; a: Id; b: Id; name_a: string; name_b: string; gender_a: Gender; gender_b: Gender; depth_a: number; depth_b: number; full: boolean; ancestor: Id | null; signature: string; path: PathNode[] }

export interface Chapter { kind: string; title: string; number: number; names: string[]; persons: Id[] }
export interface Book { chapters: Chapter[]; total: number }

export interface Issue { severity: 'error' | 'warn'; type: string; person: Id; related: Id | null; args: any[] }
export interface Completeness { criteria: string[]; missing: Record<string, Id[]>; applicable: Record<string, number>; overall: number; checked: number; scores: Record<string, number> }

export interface MergeSuggestion { a: Id; b: Id; score: number; name_a: string; name_b: string; years_a: string; years_b: string }
export interface MergeItem { value: string; label: string; in: 'A' | 'B' | 'A+B'; media: Id | null }
export interface Comparison {
  a: Person; b: Person;
  mother_a: [Id, string] | null; mother_b: [Id, string] | null; father_a: [Id, string] | null; father_b: [Id, string] | null;
  children: MergeItem[]; partners: MergeItem[]; events: MergeItem[]; gallery: MergeItem[]; clans: MergeItem[]; sources: MergeItem[];
  fields: string[];
}

export interface TreeNode {
  id: string;
  data: {
    gender: string; 'first name': string; 'last name': string; maiden_name: string; birthday: string; birth_place: string;
    death_date: string; death_place: string; years: string; avatar: Id | null; living: boolean; anon: boolean; clans: string[];
    marriages: Record<string, string>; father_rel: string; mother_rel: string; pets: { id: Id; name: string; species: string; photo: Id | null }[];
  };
  rels: { parents: string[]; spouses: string[]; children: string[] };
}

export interface GedcomReport { dry_run: boolean; created: number; updated: number; skipped: number; families: number; pets_created: number; pets_skipped: number; sources: number; places: number; warnings: [string, string][] }
export interface WpReport { persons: number; unions: number; pets: number; sources: number; media_pending: number; site: string; plugin: string; warnings: [string, string][] }
export interface WpPending { site: string; items: { wp_id: number; refs: any[] }[] }
