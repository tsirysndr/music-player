import { FC } from "react";
import { AppShell } from "../Layout";
import {
  Artwork,
  IconButton,
  Icons,
  PlayPauseButton,
  SkeletonBox,
  TrackListHeader,
  TrackRow,
  TrackSkeletonList,
  type PlaylistOption,
  type TrackRowItem,
} from "../UI";

export type AlbumDetail = {
  id: string;
  title: string;
  artist: string;
  /** A string as well as a number: some remote sources give "1999-08-31". */
  year?: number | string | null;
  cover?: string | null;
  /** "12 tracks · 48 min", assembled by the container. */
  meta?: string;
  tracks: TrackRowItem[];
};

export type AlbumDetailsProps = {
  album?: AlbumDetail;
  loading?: boolean;
  currentTrackId?: string;
  recentPlaylists: PlaylistOption[];
  onBack: () => void;
  onPlayAlbum: (albumId: string, shuffle: boolean, position?: number) => void;
  onPlayNext: (trackId: string) => void;
  onToggleLike: (trackId: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
};

/**
 * The desktop's album detail: a 150px cover beside the title block, the
 * transport at the right, then the track table.
 */
const AlbumDetails: FC<AlbumDetailsProps> = ({
  album,
  loading,
  currentTrackId,
  recentPlaylists,
  onBack,
  onPlayAlbum,
  onPlayNext,
  onToggleLike,
  onAddTrackToPlaylist,
}) => (
  <AppShell title="Album" onBack={onBack}>
    {loading || !album ? (
      <>
        <div className="flex items-end gap-5 pb-3">
          <SkeletonBox className="size-[150px]" />
          <div className="flex flex-1 flex-col gap-3">
            <SkeletonBox className="h-3 w-16" />
            <SkeletonBox className="h-7 w-1/2" />
            <SkeletonBox className="h-4 w-1/4" />
          </div>
        </div>
        <TrackSkeletonList rows={8} />
      </>
    ) : (
      <>
        <div className="flex flex-col gap-5 pb-4 sm:flex-row sm:items-end">
          <Artwork
            src={album.cover}
            alt={album.title}
            fallbackIcon={Icons.disc}
            className="size-[150px] shrink-0"
          />
          <div className="flex min-w-0 flex-1 flex-col gap-[6px]">
            <span className="text-[10px] tracking-[1.5px] text-muted">
              ALBUM
            </span>
            <h2 className="truncate text-2xl font-bold text-fg lg:text-[26px]">
              {album.title}
            </h2>
            <p className="truncate text-sm text-dim">{album.artist}</p>
            <p className="truncate text-xs text-muted">
              {[album.year, album.meta].filter(Boolean).join(" · ")}
            </p>
          </div>
          <div className="flex items-center gap-[10px]">
            <IconButton
              icon={Icons.shuffle}
              iconSize={18}
              size={44}
              aria-label="Shuffle album"
              onClick={() => onPlayAlbum(album.id, true)}
            />
            <PlayPauseButton
              aria-label="Play album"
              onClick={() => onPlayAlbum(album.id, false)}
            />
          </div>
        </div>

        <TrackListHeader />
        <div className="flex flex-col">
          {album.tracks.map((track, index) => (
            <TrackRow
              key={track.id}
              track={track}
              index={index}
              useTrackNumber
              current={track.id === currentTrackId}
              playlists={recentPlaylists}
              onPlay={() => onPlayAlbum(album.id, false, index)}
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
  </AppShell>
);

export default AlbumDetails;
