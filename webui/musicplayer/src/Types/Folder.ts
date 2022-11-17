import { Playlist } from "./Playlist";

export type Folder = {
  id: string;
  name: string;
  playlists: Playlist[];
};
