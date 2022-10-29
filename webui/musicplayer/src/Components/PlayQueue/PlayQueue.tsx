import styled from "@emotion/styled";
import { FC } from "react";
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

export type PlayQueueProps = {
  tracks: any[];
  onClickLibraryItem: (item: string) => void;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  currentIndex?: number;
  onPlayTrack: (id: string, postion?: number) => void;
};

const PlayQueue: FC<PlayQueueProps> = (props) => {
  const { onClickLibraryItem, tracks, currentIndex, nowPlaying, onPlayTrack } =
    props;
  return (
    <Container>
      <Sidebar active="play-queue" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar {...props} />
        <MainContent title="Play Queue">
          <TracksTable
            tracks={tracks}
            currentIndex={currentIndex}
            isPlaying={nowPlaying.isPlaying}
            onPlayTrack={onPlayTrack}
          />
        </MainContent>
      </Content>
    </Container>
  );
};

export default PlayQueue;
