import { FC } from "react";
import { AppShell } from "../Layout";
import {
  AlbumCard,
  Artwork,
  IconButton,
  Icons,
  PlayPauseButton,
  SectionHeader,
  SkeletonBox,
  TrackListHeader,
  TrackRow,
  type AlbumCardItem,
  type PlaylistOption,
  type TrackRowItem,
} from "../UI";

export type ArtistDetail = {
  id: string;
  name: string;
  picture?: string | null;
};

export type ArtistDetailsProps = {
  artist?: ArtistDetail;
  albums: AlbumCardItem[];
  tracks: TrackRowItem[];
  loading?: boolean;
  currentTrackId?: string;
  recentPlaylists: PlaylistOption[];
  onBack: () => void;
  onPlayArtist: (artistId: string, shuffle: boolean, position?: number) => void;
  onPlayAlbum: (albumId: string, shuffle: boolean) => void;
  onPlayNext: (trackId: string) => void;
  onToggleLike: (trackId: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
};

/** The artist page: round portrait, the artist's albums, then their songs. */
const ArtistDetails: FC<ArtistDetailsProps> = ({
  artist,
  albums,
  tracks,
  loading,
  currentTrackId,
  recentPlaylists,
  onBack,
  onPlayArtist,
  onPlayAlbum,
  onPlayNext,
  onToggleLike,
  onAddTrackToPlaylist,
}) => (
  <AppShell title="Artist" onBack={onBack}>
    {loading || !artist ? (
      <div className="flex items-end gap-5 pb-3">
        <SkeletonBox className="size-[150px] rounded-full" />
        <div className="flex flex-1 flex-col gap-3">
          <SkeletonBox className="h-3 w-16" />
          <SkeletonBox className="h-7 w-1/2" />
        </div>
      </div>
    ) : (
      <>
        <div className="flex flex-col gap-5 pb-6 sm:flex-row sm:items-end">
          <Artwork
            src={artist.picture}
            alt={artist.name}
            fallbackIcon={Icons.artist}
            rounded="full"
            className="size-[150px] shrink-0"
          />
          <div className="flex min-w-0 flex-1 flex-col gap-[6px]">
            <span className="text-[10px] tracking-[1.5px] text-muted">
              ARTIST
            </span>
            <h2 className="truncate text-2xl font-bold text-fg lg:text-[26px]">
              {artist.name}
            </h2>
            <p className="truncate text-xs text-muted">
              {`${albums.length} ${albums.length === 1 ? "album" : "albums"} · ${tracks.length} ${
                tracks.length === 1 ? "song" : "songs"
              }`}
            </p>
          </div>
          <div className="flex items-center gap-[10px]">
            <IconButton
              icon={Icons.shuffle}
              iconSize={18}
              size={44}
              aria-label={`Shuffle ${artist.name}`}
              onClick={() => onPlayArtist(artist.id, true)}
            />
            <PlayPauseButton
              aria-label={`Play ${artist.name}`}
              onClick={() => onPlayArtist(artist.id, false)}
            />
          </div>
        </div>

        {albums.length > 0 && (
          <>
            <SectionHeader title="ALBUMS" className="mb-3" />
            <div className="mb-8 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
              {albums.map((album) => (
                <AlbumCard
                  key={album.id}
                  album={album}
                  onPlay={() => onPlayAlbum(album.id, false)}
                  onShufflePlay={() => onPlayAlbum(album.id, true)}
                />
              ))}
            </div>
          </>
        )}

        {tracks.length > 0 && (
          <>
            <SectionHeader title="SONGS" className="mb-1" />
            <TrackListHeader />
            <div className="flex flex-col">
              {tracks.map((track, index) => (
                <TrackRow
                  key={track.id}
                  track={track}
                  index={index}
                  current={track.id === currentTrackId}
                  playlists={recentPlaylists}
                  onPlay={() => onPlayArtist(artist.id, false, index)}
                  onLike={() => onToggleLike(track.id)}
                  onPlayNext={() => onPlayNext(track.id)}
                  onAddToPlaylist={(playlistId) =>
                    onAddTrackToPlaylist(playlistId, track.id)
                  }
                />
              ))}
            </div>
          </>
        )}
      </>
    )}
  </AppShell>
);

export default ArtistDetails;
