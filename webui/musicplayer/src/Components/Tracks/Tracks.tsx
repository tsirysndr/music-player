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

export type TracksProps = {
  tracks: any[];
  onClickLibraryItem: (item: string) => void;
};

const Tracks: FC<TracksProps> = ({ onClickLibraryItem, tracks }) => {
  return (
    <Container>
      <Sidebar active="tracks" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar />
        <MainContent title="Tracks">
          <TracksTable tracks={tracks} />
        </MainContent>
      </Content>
    </Container>
  );
};

export default Tracks;
