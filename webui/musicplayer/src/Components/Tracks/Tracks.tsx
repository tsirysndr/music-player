import styled from "@emotion/styled";
import { FC } from "react";
import { Device } from "../../Types/Device";
import ControlBar from "../ControlBar";
import ListeningOn from "../ListeningOn";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import TracksTable from "../TracksTable";

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

export type TracksProps = {
  tracks: any[];
  onCreatePlaylist: (name: string, description?: string) => void;
  nowPlaying: any;
  onPlayTrack: (id: string, postion?: number) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onPlayNext: (id: string) => void;
  recentPlaylists: any[];
  currentCastDevice?: Device;
  onFilter: (filter: string) => void;
};

const Tracks: FC<TracksProps> = (props) => {
  const {
    tracks,
    nowPlaying,
    onPlayTrack,
    onPlayNext,
    onCreatePlaylist,
    onAddTrackToPlaylist,
    recentPlaylists,
    currentCastDevice,
    onFilter,
  } = props;
  return (
    <>
      {currentCastDevice && <ListeningOn deviceName={currentCastDevice.name} />}
      <Container>
        <Sidebar active="tracks" />
        <Content>
          <ControlBar />
          <MainContent title="Tracks" onFilter={onFilter}>
            <TracksTable
              tracks={tracks}
              currentTrackId={nowPlaying.id}
              isPlaying={nowPlaying.isPlaying}
              onPlayTrack={onPlayTrack}
              onPlayNext={onPlayNext}
              onCreatePlaylist={onCreatePlaylist}
              recentPlaylists={recentPlaylists}
              onAddTrackToPlaylist={onAddTrackToPlaylist}
            />
          </MainContent>
        </Content>
      </Container>
    </>
  );
};

export default Tracks;
