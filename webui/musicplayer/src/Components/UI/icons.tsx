/**
 * The icon set, named after `desktop/ui/icons.slint` rather than after Tabler.
 *
 * Almost every glyph *is* the desktop's own SVG: `desktopIcons.tsx` is
 * generated from `desktop/assets/icons/*.svg`, the same files the Slint client
 * embeds, so the two clients cannot drift onto different marks. Tabler fills in
 * only what the desktop has no icon for — the device speaker and the tick.
 *
 * Keeping the desktop's names means a component ported from Slint asks for the
 * same icon here as it does there: `Icons.listMusic` is `Icons.list-music`.
 */
import {
  IconCheck,
  IconDeviceSpeaker,
} from "@tabler/icons-react";
import { DESKTOP_ICONS } from "./desktopIcons";
import type { IconComponent } from "./iconTypes";

export type { IconComponent, IconProps } from "./iconTypes";

export const Icons = {
  // ── The desktop's own glyphs ────────────────────────────────────────────
  disc: DESKTOP_ICONS.disc,
  artist: DESKTOP_ICONS.artist,
  listMusic: DESKTOP_ICONS.listMusic,
  music: DESKTOP_ICONS.music,
  heart: DESKTOP_ICONS.heart,
  heartOutline: DESKTOP_ICONS.heartOutline,
  search: DESKTOP_ICONS.search,
  play: DESKTOP_ICONS.play,
  pause: DESKTOP_ICONS.pause,
  prev: DESKTOP_ICONS.prev,
  next: DESKTOP_ICONS.next,
  volume: DESKTOP_ICONS.volume,
  volumeMute: DESKTOP_ICONS.volumeMute,
  settings: DESKTOP_ICONS.settings,
  chevronLeft: DESKTOP_ICONS.chevronLeft,
  shuffle: DESKTOP_ICONS.shuffle,
  repeat: DESKTOP_ICONS.repeat,
  equalizer: DESKTOP_ICONS.equalizer,
  panelLeft: DESKTOP_ICONS.panelLeft,
  panelLeftFilled: DESKTOP_ICONS.panelLeftFilled,
  panelRight: DESKTOP_ICONS.panelRight,
  panelRightFilled: DESKTOP_ICONS.panelRightFilled,
  broadcast: DESKTOP_ICONS.broadcast,
  server: DESKTOP_ICONS.server,
  folder: DESKTOP_ICONS.folder,
  navidrome: DESKTOP_ICONS.navidrome,
  jellyfin: DESKTOP_ICONS.jellyfin,
  trash: DESKTOP_ICONS.trash,
  playlist: DESKTOP_ICONS.playlist,
  pencil: DESKTOP_ICONS.pencil,
  circlePlus: DESKTOP_ICONS.circlePlus,
  close: DESKTOP_ICONS.close,
  ellipsis: DESKTOP_ICONS.ellipsis,
  extension: DESKTOP_ICONS.extension,
  refresh: DESKTOP_ICONS.refresh,

  // ── Tabler, for what the desktop has no glyph for ───────────────────────
  device: IconDeviceSpeaker,
  connect: DESKTOP_ICONS.plugConnected,
  cast: DESKTOP_ICONS.cast,
  chromecast: DESKTOP_ICONS.chromecast,
  deviceSpeaker: DESKTOP_ICONS.deviceSpeaker,
  check: IconCheck,
} satisfies Record<string, IconComponent>;

export type IconName = keyof typeof Icons;

/** Re-exported so a story can render the whole desktop set. */
export { DESKTOP_ICONS };
