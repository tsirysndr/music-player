import styled from "@emotion/styled";
import { FC } from "react";
import Button from "../Button";
import ControlBar from "../ControlBar";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
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

const BackButton = styled.button`
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 30px;
  width: 30px;
  border-radius: 15px;
  background-color: #f7f7f8;
  margin-left: 10px;
  margin-bottom: 46px;
  position: absolute;
  z-index: 1;
`;

const AlbumCover = styled.img`
  height: 240px;
  width: 240px;
  border-radius: 3px;
  cursor: pointer;
  margin-left: 10px;
`;

const Album = styled.div`
  display: flex;
  flex-direction: row;
  margin-top: 90px;
`;

const Title = styled.div`
  font-family: RockfordSansBold;
  font-size: 32px;
`;

const Artist = styled.div`
  font-family: RockfordSansMedium;
  font-size: 14px;
  margin-top: 8px;
`;

const Metadata = styled.div`
  display: flex;
  flex-direction: column;
  margin-left: 26px;
  height: 240px;
`;

const Buttons = styled.div`
  display: flex;
  flex-direction: row;
`;

const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;
`;

const MetadataContainer = styled.div`
  display: flex;
  flex: 1;
  justify-content: center;
  flex-direction: column;
`;

const Separator = styled.div`
  width: 26px;
`;

const Label = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
`;

const Icon = styled.div`
  margin-top: 6px;
`;

const Tracks = styled.div`
  margin-top: 25px;
  font-family: RockfordSansLight;
  font-size: 14px;
`;

export type AlbumDetailsProps = {
  onClickAlbum: (album: any) => void;
  onBack: () => void;
  onClickLibraryItem: (item: string) => void;
  album: any;
};

const AlbumDetails: FC<AlbumDetailsProps> = ({
  onBack,
  onClickLibraryItem,
  album,
}) => {
  return (
    <Container>
      <Sidebar active="albums" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar />
        <MainContent displayHeader={false}>
          <Scrollable>
            <BackButton onClick={onBack}>
              <div style={{ marginTop: 2 }}>
                <ArrowBack />
              </div>
            </BackButton>
            <Album>
              <AlbumCover src={album.cover} />
              <Metadata>
                <MetadataContainer>
                  <Title>{album.title}</Title>
                  <Artist>{album.artist}</Artist>
                  <Tracks>{album.tracks.length} TRACKS</Tracks>
                </MetadataContainer>
                <Buttons>
                  <Button onClick={() => {}} kind="primary">
                    <Label>
                      <Icon>
                        <Play small color="#fff" />
                      </Icon>
                      <div style={{ marginLeft: 7 }}>Play</div>
                    </Label>
                  </Button>
                  <Separator />
                  <Button onClick={() => {}} kind="secondary">
                    <Label>
                      <Shuffle color="#ab28fc" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </Label>
                  </Button>
                </Buttons>
              </Metadata>
            </Album>
            <TracksTable
              tracks={album.tracks}
              header={["#", "Title", "Artist", "Time"]}
            />
          </Scrollable>
        </MainContent>
      </Content>
    </Container>
  );
};

export default AlbumDetails;
