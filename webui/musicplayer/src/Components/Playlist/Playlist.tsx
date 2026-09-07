import { FC } from "react";
import { AppShell } from "../Layout";
import {
  EmptyState,
  IconButton,
  Icons,
  PlayPauseButton,
  TrackListHeader,
  TrackRow,
  TrackSkeletonList,
  type PlaylistOption,
  type TrackRowItem,
} from "../UI";

export type PlaylistDetail = {
  id: string;
  name: string;
  description?: string | null;
  tracks: TrackRowItem[];
};

export type PlaylistProps = {
  playlist?: PlaylistDetail;
  loading?: boolean;
  currentTrackId?: string;
  recentPlaylists: PlaylistOption[];
  onBack: () => void;
  onPlayPlaylist: (id: string, shuffle: boolean, position?: number) => void;
  onPlayNext: (trackId: string) => void;
  onToggleLike: (trackId: string) => void;
  /** Removal is by position — the daemon's mutation takes an index, and a
      playlist may legitimately hold the same track twice. */
  onRemoveTrack: (position: number) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
};

const trackLabel = (count: number) =>
  count === 1 ? "1 track" : `${count} tracks`;

/** The desktop's playlist detail view. */
const Playlist: FC<PlaylistProps> = ({
  playlist,
  loading,
  currentTrackId,
  recentPlaylists,
  onBack,
  onPlayPlaylist,
  onPlayNext,
  onToggleLike,
  onRemoveTrack,
  onAddTrackToPlaylist,
}) => (
  <AppShell title={playlist?.name ?? "Playlist"} onBack={onBack}>
    {loading || !playlist ? (
      <TrackSkeletonList rows={10} />
    ) : (
      <>
        <div className="flex items-center gap-[10px] pb-3">
          <Icons.playlist size={22} className="shrink-0 text-accent" />
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-base font-bold text-fg">
              {playlist.name}
            </h2>
            <p className="truncate text-[11px] text-dim">
              {playlist.description
                ? `${playlist.description} · ${trackLabel(playlist.tracks.length)}`
                : trackLabel(playlist.tracks.length)}
            </p>
          </div>
          <IconButton
            icon={Icons.shuffle}
            iconSize={17}
            aria-label="Shuffle playlist"
            onClick={() => onPlayPlaylist(playlist.id, true)}
          />
          <PlayPauseButton
            aria-label="Play playlist"
            onClick={() => onPlayPlaylist(playlist.id, false)}
          />
        </div>

        {playlist.tracks.length === 0 ? (
          <EmptyState
            icon={Icons.playlist}
            title="Empty playlist"
            hint="Add tracks from the library with the “…” menu on any row."
          />
        ) : (
          <>
            <TrackListHeader />
            <div className="flex flex-col">
              {playlist.tracks.map((track, index) => (
                <TrackRow
                  key={`${track.id}-${index}`}
                  track={track}
                  index={index}
                  current={track.id === currentTrackId}
                  playlists={recentPlaylists}
                  onPlay={() => onPlayPlaylist(playlist.id, false, index)}
                  onLike={() => onToggleLike(track.id)}
                  onPlayNext={() => onPlayNext(track.id)}
                  onRemove={() => onRemoveTrack(index)}
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

export default Playlist;
