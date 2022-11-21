import styled from "@emotion/styled";
import { FC } from "react";
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
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  onPlayTrack: (id: string, postion?: number) => void;
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  nextTracks: Track[];
  previousTracks: Track[];
  onPlayNext: (id: string) => void;
  onPlayTrackAt: (position: number) => void;
  onRemoveTrackAt: (position: number) => void;
  onSearch: (query: string) => void;
  onPlayPlaylist: (
    playlistId: string,
    shuffle: boolean,
    position?: number
  ) => void;
  recentPlaylists: any[];
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
  } = props;
  return (
    <Container>
      <Sidebar active="tracks" {...props} />
      <Content>
        <ControlBar {...props} />
        <MainContent title="Tracks">
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
  );
};

export default Tracks;
