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
  onClickLibraryItem: (item: string) => void;
};

const PlayQueue: FC<PlayQueueProps> = ({ onClickLibraryItem }) => {
  return (
    <Container>
      <Sidebar active="play-queue" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar />
        <MainContent title="Play Queue">
          <TracksTable />
        </MainContent>
      </Content>
    </Container>
  );
};

export default PlayQueue;
