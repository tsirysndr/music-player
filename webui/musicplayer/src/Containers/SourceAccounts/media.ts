import gql from "graphql-tag";
import {
  BROWSE_SOURCE_MEDIA, type BrowseSourceMediaQuery, type BrowseSourceMediaVariables,
  type SourceMediaTrack,
} from "../../GraphQL/SourceAccounts/Query";
import type { AudioChoice } from "../Media/api";
import { sourceRequest, UNCERTAIN } from "./requests";

const PLAY_SELECTED_SOURCE_MEDIA = gql`
  mutation PlaySelectedSourceMedia($entry: SelectedMediaInput!, $next: Boolean!) {
    playSelectedMedia(entry: $entry, next: $next)
  }
`;

export async function playSourceTrack(track: SourceMediaTrack, next: boolean, signal: AbortSignal, choice?: AudioChoice) {
  const result = await sourceRequest<{ playSelectedMedia: string }>(PLAY_SELECTED_SOURCE_MEDIA,
    { entry: { track: sourceTrackInput(track), ...(choice === undefined ? {} : { choice }) }, next }, signal, true);
  if (typeof result.playSelectedMedia !== "string" || !result.playSelectedMedia.trim()) {
    throw new Error("Unexpected playback acknowledgement. " + UNCERTAIN);
  }
  return result.playSelectedMedia;
}

const ADD_SOURCE_TRACKS = gql`
  mutation AddSourceTracks($tracks: [TrackInput!]!) { addTracks(tracks: $tracks) }
`;
const ADD_SELECTED_SOURCE_MEDIA = gql`
  mutation AddSelectedSourceMedia($entries: [SelectedMediaInput!]!) {
    addSelectedMedia(entries: $entries)
  }
`;

// No fabricated album/artist identity and no temporary stream URL. These are
// the actual browse tags required by v1, whose ID-only lookup is unsupported.
export function sourceTrackInput(track: SourceMediaTrack) {
  const { id, title, artist, uri, duration, discNumber, trackNumber, album } = track;
  if (id !== uri || !(id.startsWith("mp-source:v1?") || id.startsWith("mp-source:v2?"))) {
    throw new Error("The source returned a track without a stable source handle");
  }
  return { id, title, artist, uri, duration, discNumber, trackNumber,
    albumTitle: album.title, albumArtist: album.artist, albumYear: album.year };
}

function decimal(value: string) {
  return /^(0|[1-9][0-9]*)$/.test(value) &&
    (value.length < 20 || (value.length === 20 && value <= "18446744073709551615"));
}
function advances(next: string, offset: string) {
  return decimal(next) && (next.length > offset.length || (next.length === offset.length && next > offset));
}

export async function readSourcePage(variables: BrowseSourceMediaVariables, signal: AbortSignal) {
  const offset = variables.offset ?? "0";
  if (!decimal(offset)) throw new Error("Invalid source page offset");
  const result = await sourceRequest<BrowseSourceMediaQuery>(BROWSE_SOURCE_MEDIA, variables, signal);
  const page = result.browseSourceMedia;
  if (!page || page.accountId !== variables.accountId) throw new Error("The source response belongs to another account");
  if (page.entries.length > (variables.limit ?? 100) ||
      (page.nextOffset !== null && !advances(page.nextOffset, offset))) {
    throw new Error("The source returned invalid pagination");
  }
  return page;
}

// Every page of this view, including a legacy final empty page. Containers
// remain navigation targets; this does not guess recursive list semantics.
export async function readAllViewTracks(variables: BrowseSourceMediaVariables, signal: AbortSignal) {
  const tracks: SourceMediaTrack[] = [];
  let offset = "0";
  for (;;) {
    signal.throwIfAborted();
    const page = await readSourcePage({ ...variables, offset }, signal);
    for (const entry of page.entries) {
      if (!entry.isContainer && entry.track) tracks.push(entry.track);
    }
    if (page.nextOffset === null) return tracks;
    offset = page.nextOffset;
  }
}

export async function enqueueSourceTracks(tracks: SourceMediaTrack[], signal: AbortSignal) {
  const inputs = tracks.map(sourceTrackInput);
  const result = await sourceRequest<{ addTracks: boolean }>(ADD_SOURCE_TRACKS, { tracks: inputs }, signal, true);
  if (!result.addTracks) throw new Error("The queue update was not accepted");
}

export async function enqueueSelectedSource(track: SourceMediaTrack, choice: AudioChoice, signal: AbortSignal) {
  const result = await sourceRequest<{ addSelectedMedia: string[] }>(ADD_SELECTED_SOURCE_MEDIA,
    { entries: [{ track: sourceTrackInput(track), choice }] }, signal, true);
  if (result.addSelectedMedia.length !== 1) throw new Error("Unexpected queue acknowledgement. " + UNCERTAIN);
}
