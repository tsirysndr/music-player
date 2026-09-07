import { atom } from "jotai";

/**
 * Chrome state shared by the shell — the same flags the desktop client keeps
 * on its `AppWindow` (`show-sidebar`, `show-queue`, `show-palette`,
 * `show-full-player`), so the two behave alike under the same shortcuts.
 */
export const sidebarOpenAtom = atom(true);
export const queueOpenAtom = atom(false);
export const paletteOpenAtom = atom(false);
export const fullPlayerOpenAtom = atom(false);
/** The "More" sheet behind the last tab of the mobile bottom bar. */
export const mobileMenuOpenAtom = atom(false);
/** The audio-settings modal — the desktop's `show-audio`, opened with `e`. */
export const audioSettingsOpenAtom = atom(false);
