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
  onClickLibraryItem: (item: string) => void;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  onPlayTrack: (id: string, postion?: number) => void;
  nextTracks: Track[];
  previousTracks: Track[];
  onPlayNext: (id: string) => void;
  onPlayTrackAt: (position: number) => void;
  onRemoveTrackAt: (position: number) => void;
};

const Tracks: FC<TracksProps> = (props) => {
  const { onClickLibraryItem, tracks, nowPlaying, onPlayTrack, onPlayNext } =
    props;
  return (
    <Container>
      <Sidebar active="tracks" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar {...props} />
        <MainContent title="Tracks">
          <TracksTable
            tracks={tracks}
            currentTrackId={nowPlaying.id}
            isPlaying={nowPlaying.isPlaying}
            onPlayTrack={onPlayTrack}
            onPlayNext={onPlayNext}
          />
        </MainContent>
      </Content>
    </Container>
  );
};

export default Tracks;
