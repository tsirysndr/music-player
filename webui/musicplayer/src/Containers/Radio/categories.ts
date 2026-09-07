import { Icons, type IconComponent } from "../../Components/UI";

export type RadioCategory = {
  label: string;
  term: string;
  icon: IconComponent;
  /** Fixed per category rather than skin-derived, exactly as the desktop's
      `RadioCategoryTile` calls are written — the hue is what tells the tiles
      apart at a glance. */
  color: string;
};

/** The category grid, matching `desktop/ui/app.slint` tile for tile. */
export const RADIO_CATEGORIES: RadioCategory[] = [
  { label: "Synthwave", term: "synthwave", icon: Icons.equalizer, color: "#ff4fa3" },
  { label: "Lo-fi", term: "lofi", icon: Icons.volume, color: "#39d9e8" },
  { label: "Jazz", term: "jazz", icon: Icons.music, color: "#f2c94c" },
  { label: "Techno", term: "techno", icon: Icons.volume, color: "#9b6cff" },
  { label: "Ambient", term: "ambient", icon: Icons.equalizer, color: "#39d9e8" },
  { label: "Classical", term: "classical", icon: Icons.music, color: "#f052d4" },
  { label: "Rock", term: "rock", icon: Icons.disc, color: "#ff4fa3" },
  { label: "Pop", term: "pop", icon: Icons.artist, color: "#f052d4" },
  { label: "Electronic", term: "electronic", icon: Icons.disc, color: "#4d8dff" },
  { label: "Hip-Hop", term: "hip hop", icon: Icons.disc, color: "#f2c94c" },
  { label: "Chillout", term: "chill", icon: Icons.heart, color: "#39d9e8" },
  { label: "Dance", term: "dance", icon: Icons.volume, color: "#ff4fa3" },
  { label: "Reggae", term: "reggae", icon: Icons.music, color: "#f2c94c" },
  { label: "Metal", term: "metal", icon: Icons.disc, color: "#9b6cff" },
  { label: "News", term: "news", icon: Icons.broadcast, color: "#39d9e8" },
  { label: "World", term: "world", icon: Icons.broadcast, color: "#4d8dff" },
  { label: "House", term: "house", icon: Icons.volume, color: "#39d9e8" },
  { label: "Trance", term: "trance", icon: Icons.equalizer, color: "#9b6cff" },
  { label: "Drum & Bass", term: "drum and bass", icon: Icons.volume, color: "#4d8dff" },
  { label: "Disco", term: "disco", icon: Icons.disc, color: "#ff4fa3" },
  { label: "Funk", term: "funk", icon: Icons.disc, color: "#f052d4" },
  { label: "Soul", term: "soul", icon: Icons.music, color: "#ff4fa3" },
  { label: "R&B", term: "r&b", icon: Icons.artist, color: "#9b6cff" },
  { label: "Blues", term: "blues", icon: Icons.music, color: "#4d8dff" },
  { label: "Country", term: "country", icon: Icons.music, color: "#f2c94c" },
  { label: "Folk", term: "folk", icon: Icons.music, color: "#39d9e8" },
  { label: "Punk", term: "punk", icon: Icons.disc, color: "#ff4fa3" },
  { label: "Indie", term: "indie", icon: Icons.volume, color: "#f052d4" },
  { label: "Latin", term: "latin", icon: Icons.music, color: "#f052d4" },
  { label: "K-Pop", term: "k-pop", icon: Icons.artist, color: "#ff4fa3" },
  { label: "Gospel", term: "gospel", icon: Icons.music, color: "#f2c94c" },
  { label: "Oldies", term: "oldies", icon: Icons.disc, color: "#39d9e8" },
  { label: "Soundtrack", term: "soundtrack", icon: Icons.playlist, color: "#4d8dff" },
];
