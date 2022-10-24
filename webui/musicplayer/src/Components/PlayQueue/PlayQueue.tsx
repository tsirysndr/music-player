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
};

const PlayQueue: FC<PlayQueueProps> = (props) => {
  const { onClickLibraryItem, tracks } = props;
  return (
    <Container>
      <Sidebar active="play-queue" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar {...props} />
        <MainContent title="Play Queue">
          <TracksTable tracks={tracks} />
        </MainContent>
      </Content>
    </Container>
  );
};

export default PlayQueue;
