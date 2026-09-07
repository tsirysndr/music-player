import { useEffect, useMemo, useState } from "react";
import { fetcher } from "../../Api/fetcher";
import { AppShell } from "../../Components/Layout";
import {
  EmptyState,
  IconButton,
  Icons,
  PageToolbar,
  PlayPauseButton,
  TrackListHeader,
  TrackRow,
  TrackSkeletonList,
} from "../../Components/UI";
import { useTimeFormat } from "../../Hooks/useFormat";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { usePlayTrack } from "../../Hooks/usePlayTrack";

type LikedTrack = {
  id: string;
  title: string;
  artist: string;
  duration?: number;
  artists: { id: string }[];
  album: { id: string; title: string; cover?: string };
};

const query = `query($offset:Int,$limit:Int){
  likedTracks(offset:$offset,limit:$limit){
    id title artist duration
    artists { id }
    album { id title cover }
  }
}`;

/**
 * Songs liked on Rocksky that resolved to a file in the local library. The
 * likes are imported from the user's atproto repo by the daemon; this page
 * only reads the result, so it is empty until an account is linked.
 */
export default function LikedPage() {
  const [tracks, setTracks] = useState<LikedTrack[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [filter, setFilter] = useState("");
  const { formatTime } = useTimeFormat();
  const { nowPlaying, playNext } = usePlayback();
  const playTrack = usePlayTrack();
  const { isLiked, toggleLike } = useLikes();
  const { recentPlaylists, addTrackToPlaylist } = usePlaylist();

  useEffect(() => {
    let active = true;
    fetcher<any, any>(query, { offset: 0, limit: 500 })()
      .then((data) => {
        if (active) setTracks(data.likedTracks || []);
      })
      .catch((e) => {
        if (active)
          setError(
            e instanceof Error ? e.message : "Unable to load liked songs"
          );
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  const rows = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    return tracks
      .filter(
        (track) =>
          !needle ||
          [track.title, track.artist, track.album.title].some((value) =>
            value?.toLowerCase().includes(needle)
          )
      )
      .map((track) => ({
        id: track.id,
        title: track.title,
        artist: track.artist,
        artistId: track.artists[0]?.id,
        album: track.album.title,
        albumId: track.album.id,
        duration: formatTime((track.duration || 0) * 1000),
        liked: isLiked(track.id),
      }));
  }, [tracks, filter, formatTime, isLiked]);

  const playAll = (shuffle: boolean) => {
    if (rows.length === 0) return;
    const pick = shuffle ? Math.floor(Math.random() * rows.length) : 0;
    playTrack(rows[pick].id);
  };

  return (
    <AppShell>
      <PageToolbar
        filter={filter}
        filterPlaceholder="Filter liked…"
        onFilter={tracks.length > 0 ? setFilter : undefined}
      >
        {rows.length > 0 && (
          <div className="flex items-center gap-2">
            <IconButton
              icon={Icons.shuffle}
              iconSize={16}
              size={38}
              aria-label="Shuffle liked tracks"
              onClick={() => playAll(true)}
            />
            <PlayPauseButton
              size={38}
              aria-label="Play liked tracks"
              onClick={() => playAll(false)}
            />
          </div>
        )}
      </PageToolbar>

      {loading ? (
        <TrackSkeletonList rows={8} />
      ) : error ? (
        <EmptyState
          icon={Icons.heartOutline}
          title="Unable to load liked songs"
          hint={error}
        />
      ) : rows.length === 0 ? (
        <EmptyState
          icon={Icons.heartOutline}
          title={filter ? `Nothing matches “${filter}”` : "No liked song yet"}
          hint={
            filter
              ? undefined
              : "Songs you like on Rocksky show up here once they match a track in your library."
          }
        />
      ) : (
        <>
          <TrackListHeader />
          <div className="flex flex-col">
            {rows.map((track, index) => (
              <TrackRow
                key={track.id}
                track={track}
                index={index}
                current={
                  nowPlaying?.isPlaying && track.id === nowPlaying.id
                }
                playlists={recentPlaylists}
                onPlay={() => playTrack(track.id)}
                onLike={() => toggleLike(track.id)}
                onPlayNext={() => playNext({ trackId: track.id })}
                onAddToPlaylist={(playlistId) =>
                  addTrackToPlaylist({ playlistId, trackId: track.id })
                }
              />
            ))}
          </div>
        </>
      )}
    </AppShell>
  );
}
