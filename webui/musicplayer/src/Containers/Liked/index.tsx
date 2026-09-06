import styled from "@emotion/styled";
import { useTheme } from "@emotion/react";
import { useEffect, useMemo, useState } from "react";
import ContentLoader from "react-content-loader";
import ControlBar from "../../Components/ControlBar";
import MainContent from "../../Components/MainContent";
import Sidebar from "../../Components/Sidebar";
import TracksTable from "../../Components/TracksTable";
import { fetcher } from "../../Api/fetcher";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import ListeningOn from "../../Components/ListeningOn";
import { resourceUriResolver } from "../../ResourceUriResolver";

type LikedTrack = {
  id: string;
  title: string;
  artist: string;
  duration?: number;
  artists: { id: string }[];
  album: { id: string; title: string; cover?: string };
};

const Container = styled.div`
  display: flex;
  flex-direction: row;
  background-color: ${(props) => props.theme.colors.background};
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

// The document does not scroll (index.css pins body overflow); every page
// brings its own scroll container.
const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;

  &::-webkit-scrollbar {
    display: none;
  }
  scrollbar-width: none;
  -ms-overflow-style: none;
`;

const Message = styled.div`
  padding: 24px 26px;
  color: ${(props) => props.theme.colors.secondaryText};
`;

const LoaderWrapper = styled.div`
  padding: 0 26px;
`;

/// Skeleton rows, in the theme's loader colors like everywhere else.
const TracksLoader = () => {
  const theme = useTheme();
  return (
    <LoaderWrapper>
      {Array.from({ length: 8 }).map((_, i) => (
        <ContentLoader
          key={i}
          speed={1.6}
          width="100%"
          height={44}
          viewBox="0 0 900 44"
          backgroundColor={theme.colors.loaderBackground}
          foregroundColor={theme.colors.loaderForeground}
        >
          <rect x="0" y="14" rx="4" ry="4" width="38%" height="14" />
          <rect x="45%" y="14" rx="4" ry="4" width="22%" height="14" />
          <rect x="72%" y="14" rx="4" ry="4" width="18%" height="14" />
        </ContentLoader>
      ))}
    </LoaderWrapper>
  );
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
  const { currentCastDevice } = useDevices();
  const { recentPlaylists, createPlaylist, addTrackToPlaylist } = usePlaylist();

  useEffect(() => {
    let active = true;
    fetcher<any, any>(query, { offset: 0, limit: 500 })()
      .then((data) => {
        if (active) setTracks(data.likedTracks || []);
      })
      .catch((e) => {
        if (active)
          setError(e instanceof Error ? e.message : "Unable to load liked songs");
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
        album: track.album.title,
        time: formatTime((track.duration || 0) * 1000),
        cover: track.album.cover
          ? resourceUriResolver.resolve(`/covers/${track.album.cover}`)
          : undefined,
        artistId: track.artists[0]?.id,
        albumId: track.album.id,
      }));
  }, [tracks, filter, formatTime]);

  return (
    <>
      {currentCastDevice && <ListeningOn deviceName={currentCastDevice.name} />}
      <Container>
        <Sidebar active="liked" />
        <Content>
          <ControlBar />
          <Scrollable>
            <MainContent
              title="Liked"
              placeholder="Filter Liked"
              displayHeader={!loading && !error && tracks.length > 0}
              onFilter={setFilter}
            >
            {loading && <TracksLoader />}
            {!loading && error && <Message>{error}</Message>}
            {!loading && !error && rows.length === 0 && (
              <Message>
                No liked song yet. Songs you like on Rocksky show up here once
                they match a track in your library.
              </Message>
            )}
            {!loading && !error && rows.length > 0 && (
              <TracksTable
                tracks={rows}
                currentTrackId={nowPlaying.id}
                isPlaying={nowPlaying.isPlaying}
                onPlayTrack={() => {}}
                onPlayNext={(trackId: string) => playNext({ trackId })}
                onCreatePlaylist={(name: string, description?: string) =>
                  createPlaylist({ name, description })
                }
                recentPlaylists={recentPlaylists}
                onAddTrackToPlaylist={(playlistId: string, trackId: string) =>
                  addTrackToPlaylist({ playlistId, trackId })
                }
              />
              )}
            </MainContent>
          </Scrollable>
        </Content>
      </Container>
    </>
  );
}
