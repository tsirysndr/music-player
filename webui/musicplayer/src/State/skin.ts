import { atom } from "jotai";

/**
 * The skins the desktop client ships, in the order its `s` shortcut cycles
 * them (see `desktop/src/skin.rs`). `id` is the `data-skin` value the token
 * blocks in `styles/skins.css` key off; `name` is what both clients display.
 */
export const SKINS = [
  { id: "synthwave", name: "Synthwave" },
  { id: "late-night", name: "Late Night" },
  { id: "neutron", name: "Neutron" },
  { id: "lunar", name: "Lunar" },
  { id: "porcelain", name: "Porcelain" },
] as const;

export type SkinId = (typeof SKINS)[number]["id"];

/** Where the choice persists, mirroring the desktop's `desktop-skin` file. */
export const SKIN_STORAGE_KEY = "music-player.skin";

export const DEFAULT_SKIN: SkinId = "late-night";

export function isSkinId(value: unknown): value is SkinId {
  return SKINS.some((skin) => skin.id === value);
}

/**
 * The skin saved from a previous visit. Read eagerly so the first paint is
 * already in the right palette — a frame of the default skin before the
 * effect runs reads as a flash of the wrong colours.
 */
export function storedSkin(): SkinId {
  if (typeof window === "undefined") return DEFAULT_SKIN;
  const saved = window.localStorage.getItem(SKIN_STORAGE_KEY);
  return isSkinId(saved) ? saved : DEFAULT_SKIN;
}

export const skinAtom = atom<SkinId>(storedSkin());
