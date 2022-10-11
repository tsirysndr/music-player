import styled from "@emotion/styled";
import { FC } from "react";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

export type ArtistsProps = {
  onClickLibraryItem: (item: string) => void;
};

const Artists: FC<ArtistsProps> = ({ onClickLibraryItem }) => {
  return (
    <Container>
      <Sidebar active="artists" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar />
        <MainContent title="Artists" placeholder="Filter Artists"></MainContent>
      </Content>
    </Container>
  );
};

export default Artists;
