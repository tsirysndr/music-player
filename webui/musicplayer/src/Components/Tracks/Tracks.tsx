import styled from "@emotion/styled";
import { FC, useMemo } from "react";
import { usePlayback } from "../../Hooks/usePlayback";
import { Track } from "../../Types";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import TracksTable from "../TracksTable";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

export type TracksProps = {
  tracks: any[];
  folders: any[];
  playlists: any[];
  onCreateFolder: (name: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onDeleteFolder: (id: string) => void;
  onDeletePlaylist: (id: string) => void;
  onEditFolder: (id: string, name: string) => void;
  onEditPlaylist: (id: string, name: string, description?: string) => void;
  onClickLibraryItem: (item: string) => void;
  onPlayTrack: (id: string, postion?: number) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onSearch: (query: string) => void;
  recentPlaylists: any[];
};

const Tracks: FC<TracksProps> = (props) => {
  const {
    tracks,
    onPlayTrack,
    onCreatePlaylist,
    onAddTrackToPlaylist,
    recentPlaylists,
  } = props;
  const { currentTrackId, isPlaying, playNext, playPlaylist } = usePlayback();
  const onPlayNext = (trackId: string) => playNext({ variables: { trackId } });
  const onPlayPlaylist = (
    playlistId: string,
    shuffle: boolean,
    position?: number
  ) => playPlaylist({ variables: { playlistId, position, shuffle } });
  const Memoized = useMemo(
    () => (
      <Container>
        <Sidebar active="tracks" {...props} onPlayPlaylist={onPlayPlaylist} />
        <Content>
          <ControlBar />
          <MainContent title="Tracks">
            <TracksTable
              tracks={tracks}
              currentTrackId={currentTrackId}
              isPlaying={isPlaying}
              onPlayTrack={onPlayTrack}
              onPlayNext={onPlayNext}
              onCreatePlaylist={onCreatePlaylist}
              recentPlaylists={recentPlaylists}
              onAddTrackToPlaylist={onAddTrackToPlaylist}
              maxHeight={"calc(100vh - 98px)"}
            />
          </MainContent>
        </Content>
      </Container>
    ),
    [tracks, currentTrackId, isPlaying]
  );
  return Memoized;
};

export default Tracks;
