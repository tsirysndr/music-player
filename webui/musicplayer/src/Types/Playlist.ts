import { Track } from "./Track";

export type Playlist = {
  id: string;
  name: string;
  desscription?: string | null;
  tracks?: Track[];
};
