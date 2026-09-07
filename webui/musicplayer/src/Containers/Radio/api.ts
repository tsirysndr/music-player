import { fetcher } from "../../Api/fetcher";

export type Station = {
  id: string;
  name: string;
  streamUrl: string;
  source: string;
  genre: string;
  country: string;
  logo: string;
  bitrate: number;
};

/** The station fields every radio query and mutation selects. */
export const STATION_FIELDS = `id name streamUrl source genre country logo bitrate`;

export async function gql<T = any>(query: string, variables: any = {}) {
  return fetcher<T, any>(query, variables)();
}
