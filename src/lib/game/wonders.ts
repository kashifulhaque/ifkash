import * as THREE from 'three';
import { sections, name, tagline, type LootItem } from '$lib/content';
import { biomeById, type BiomeId } from './biomes';
import { offsetDir } from './planet';
import { type WonderId } from './progress';

export type WonderMarker = 'campfire' | 'cabin' | 'mailbox' | 'signpost' | 'lighthouse' | 'forge' | 'aurora';

export type Wonder = {
  id: WonderId;
  /** Verb phrase shown on the interaction card, for example "Tend the fire". */
  action: string;
  biome: BiomeId;
  /** Offset from the biome centre in surface units: [east, north]. */
  offset: [number, number];
  /** Facing, radians from north. */
  yaw: number;
  marker: WonderMarker;
  /** Panel heading and short blurb. */
  title: string;
  blurb: string;
  items: LootItem[];
};

function itemsOf(id: string): LootItem[] {
  return sections.find((s) => s.id === id)?.items ?? [];
}

export const WONDERS: Wonder[] = [
  {
    id: 'about',
    action: 'Tend the fire',
    biome: 'forest',
    offset: [0, 0],
    yaw: 0.6,
    marker: 'campfire',
    title: 'Hello, wanderer',
    blurb: 'A warm fire, a small planet, and a few stories worth telling.',
    items: [
      {
        title: name,
        subtitle: 'ML Engineer · Bangalore, India',
        body: tagline,
        links: [
          { label: 'Home', url: '/' },
          { label: 'Work', url: '/work' },
          { label: 'Projects', url: '/projects' }
        ]
      }
    ]
  },
  {
    id: 'work',
    action: 'Knock on the cabin',
    biome: 'forest',
    offset: [-6.5, 4.5],
    yaw: 2.6,
    marker: 'cabin',
    title: 'Work',
    blurb: 'The places where the day job happened.',
    items: itemsOf('work')
  },
  {
    id: 'contact',
    action: 'Check the mailbox',
    biome: 'farm',
    offset: [4, -3],
    yaw: -0.8,
    marker: 'mailbox',
    title: 'Contact',
    blurb: 'Letters welcome. Carrier pigeons too.',
    items: itemsOf('contact')
  },
  {
    id: 'resume',
    action: 'Read the signpost',
    biome: 'farm',
    offset: [-5, -3.5],
    yaw: 0.4,
    marker: 'signpost',
    title: 'Resume',
    blurb: 'The whole journey on a single page.',
    items: itemsOf('resume')
  },
  {
    id: 'blog',
    action: 'Light the lighthouse',
    biome: 'shore',
    offset: [0, 0],
    yaw: 0,
    marker: 'lighthouse',
    title: 'Blog',
    blurb: 'Notes that light the way for the next traveller.',
    items: itemsOf('blog')
  },
  {
    id: 'projects',
    action: 'Stoke the forge',
    biome: 'ember',
    offset: [5.5, -2],
    yaw: -2.2,
    marker: 'forge',
    title: 'Projects',
    blurb: 'Things hammered out of spare evenings.',
    items: itemsOf('projects')
  },
  {
    id: 'education',
    action: 'Wake the aurora',
    biome: 'arctic',
    offset: [0, 0],
    yaw: 0,
    marker: 'aurora',
    title: 'Education',
    blurb: 'Where the long nights of study paid off.',
    items: itemsOf('education')
  }
];

/** Unit direction of a wonder on the sphere. */
export function wonderDir(w: Wonder, out = new THREE.Vector3()): THREE.Vector3 {
  out.copy(biomeById(w.biome).center);
  return offsetDir(out, w.offset[0], w.offset[1]);
}
