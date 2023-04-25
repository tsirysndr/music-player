import {Album, Track} from '.';

export type Artist = {
  id: string;
  name: string;
  picture?: string;
  albums?: Album[];
  tracks?: Track[];
};
