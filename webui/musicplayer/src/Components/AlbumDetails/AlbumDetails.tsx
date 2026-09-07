import { FC } from "react";
import { AppShell } from "../Layout";
import {
  Artwork,
  ContextMenu,
  ContextMenuItem,
  ContextMenuSeparator,
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
  /** Queue the whole album — `-2` plays it next, `-3` appends it. */
  onQueueAlbum?: (albumId: string, position: -2 | -3) => void;
  onLikeAlbum?: (albumId: string) => void;
};

/**
 * The album's tracks, split into discs when there is more than one.
 *
 * A single-disc album gets one unlabelled group, so the common case looks
 * exactly as it did — the same rule the desktop applies before it inserts its
 * `DISC n` rows.
 */
const byDisc = (tracks: TrackRowItem[]) => {
  const discs = [...new Set(tracks.map((track) => track.discNumber || 1))].sort(
    (a, b) => a - b
  );
  if (discs.length < 2) return [{ disc: undefined, tracks }];
  return discs.map((disc) => ({
    disc,
    tracks: tracks.filter((track) => (track.discNumber || 1) === disc),
  }));
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
  onQueueAlbum,
  onLikeAlbum,
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
            <ContextMenu
              width={190}
              trigger={
                <IconButton
                  icon={Icons.ellipsis}
                  iconSize={18}
                  size={44}
                  aria-label="Album menu"
                />
              }
            >
              <ContextMenuItem
                icon={Icons.shuffle}
                label="Shuffle play"
                onClick={() => onPlayAlbum(album.id, true)}
              />
              <ContextMenuItem
                icon={Icons.next}
                label="Play next"
                onClick={() => onQueueAlbum?.(album.id, -2)}
              />
              <ContextMenuItem
                icon={Icons.listMusic}
                label="Add to queue"
                onClick={() => onQueueAlbum?.(album.id, -3)}
              />
              <ContextMenuSeparator />
              <ContextMenuItem
                icon={Icons.heart}
                label="Like album"
                onClick={() => onLikeAlbum?.(album.id)}
              />
            </ContextMenu>
          </div>
        </div>

        <TrackListHeader />
        <div className="flex flex-col">
          {byDisc(album.tracks).map((group) => (
            <div key={group.disc ?? "all"} className="flex flex-col">
              {group.disc !== undefined && (
                <div className="flex h-9 items-center gap-2 px-3 pt-2">
                  <Icons.disc size={12} className="shrink-0 text-muted" />
                  <span className="text-[11px] tracking-[1.5px] text-muted">
                    {`DISC ${group.disc}`}
                  </span>
                </div>
              )}
              {group.tracks.map((track) => {
                // The position handed to `onPlayAlbum` indexes the whole
                // album, not the disc — the queue is the album.
                const index = album.tracks.indexOf(track);
                return (
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
                );
              })}
            </div>
          ))}
        </div>
      </>
    )}
  </AppShell>
);

export default AlbumDetails;
