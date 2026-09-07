import { FC } from "react";
import { AppShell } from "../Layout";
import {
  EmptyState,
  Icons,
  LoadMore,
  PageToolbar,
  TrackListHeader,
  TrackRow,
  TrackSkeletonList,
  type PlaylistOption,
  type TrackRowItem,
} from "../UI";

export type TracksProps = {
  tracks: TrackRowItem[];
  loading?: boolean;
  currentTrackId?: string;
  filter: string;
  recentPlaylists: PlaylistOption[];
  onFilter: (filter: string) => void;
  onPlayTrack: (id: string) => void;
  onPlayNext: (id: string) => void;
  onToggleLike: (id: string) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onLoadMore?: () => void;
  hasMore?: boolean;
};

/** The desktop's "All tracks" tab: a filter box over the shared track table. */
const Tracks: FC<TracksProps> = ({
  tracks,
  loading,
  currentTrackId,
  filter,
  recentPlaylists,
  onFilter,
  onPlayTrack,
  onPlayNext,
  onToggleLike,
  onAddTrackToPlaylist,
  onLoadMore,
  hasMore,
}) => (
  <AppShell>
    <PageToolbar
      filter={filter}
      filterPlaceholder="Filter tracks…"
      onFilter={onFilter}
    />

    {loading && tracks.length === 0 ? (
      <TrackSkeletonList />
    ) : tracks.length === 0 ? (
      <EmptyState
        icon={Icons.music}
        title={
          filter ? `Nothing matches “${filter}”` : "No tracks in the library yet"
        }
        hint={
          filter
            ? undefined
            : "Scan a folder with `music-player scan` and they will show up here."
        }
      />
    ) : (
      <>
        <TrackListHeader />
        <div className="flex flex-col">
          {tracks.map((track, index) => (
            <TrackRow
              key={track.id}
              track={track}
              index={index}
              current={track.id === currentTrackId}
              playlists={recentPlaylists}
              onPlay={() => onPlayTrack(track.id)}
              onLike={() => onToggleLike(track.id)}
              onPlayNext={() => onPlayNext(track.id)}
              onAddToPlaylist={(playlistId) =>
                onAddTrackToPlaylist(playlistId, track.id)
              }
            />
          ))}
        </div>
        <LoadMore hasMore={hasMore} onLoadMore={onLoadMore} />
      </>
    )}
  </AppShell>
);

export default Tracks;
