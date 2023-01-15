import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC } from "react";
import { Link } from "react-router-dom";
import Button from "../Button";
import ControlBar from "../ControlBar";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import TracksTable from "../TracksTable";
import AlbumIcon from "../Icons/AlbumCover";
import { Track } from "../../Types";
import { Device } from "../../Types/Device";
import { useTheme } from "@emotion/react";

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

const BackButton = styled.button`
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 30px;
  width: 30px;
  border-radius: 15px;
  background-color: ${(props) => props.theme.colors.backButton};
  margin-left: 26px;
  margin-bottom: 46px;
  position: absolute;
  z-index: 1;
`;

const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;
`;

const Artist = styled.div`
  font-family: RockfordSansBold;
  font-size: 32px;
  margin-top: 94px;
  margin-left: 26px;
  margin-bottom: 40px;
  color: ${(props) => props.theme.colors.text};
`;

const Buttons = styled.div`
  display: flex;
  flex-direction: row;
  margin-left: 26px;
  margin-bottom: 20px;
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

const Title = styled.div`
  font-family: RockfordSansBold;
  font-size: 16px;
  margin-left: 15px;
  margin-bottom: 10px;
  flex: 1;
  color: ${(props) => props.theme.colors.text};
`;

const Row = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  margin-right: 50px;
  margin-left: 10px;
`;

const SeeMore = styled.div`
  width: 64px;
  font-size: 14px;
  color: ${(props) => props.theme.colors.secondaryText};
  cursor: pointer;
`;

const Tracks = styled.div`
  margin-bottom: 48px;
`;

const AlbumCover = styled.img`
  height: 169px;
  width: 169px;
  border-radius: 5px;
  cursor: pointer;
`;

const NoAlbumCover = styled.div`
  height: 169px;
  width: 169px;
  border-radius: 5px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #ddaefb14;
`;

const AlbumArtist = styled.div`
  color: #828282;
  margin-bottom: 56px;
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
`;

const AlbumTitle = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  color: ${(props) => props.theme.colors.text};
`;

export type ArtistDetailsProps = {
  onBack: () => void;
  onClickLibraryItem: (item: string) => void;
  artist: any;
  tracks: any[];
  albums: any[];
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  nextTracks: Track[];
  previousTracks: Track[];
  onPlayArtistTracks: (
    artistId: string,
    shuffle: boolean,
    position?: number
  ) => void;
  onPlayNext: (id: string) => void;
  onPlayTrackAt: (position: number) => void;
  onRemoveTrackAt: (position: number) => void;
  onSearch: (query: string) => void;
  folders: any[];
  playlists: any[];
  onCreateFolder: (name: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onDeleteFolder: (id: string) => void;
  onDeletePlaylist: (id: string) => void;
  onEditFolder: (id: string, name: string) => void;
  onEditPlaylist: (id: string, name: string, description?: string) => void;
  recentPlaylists: any[];
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onPlayPlaylist: (
    playlistId: string,
    shuffle: boolean,
    position?: number
  ) => void;
  devices: Device[];
  castDevices: Device[];
  currentDevice?: Device;
  connectToDevice: (deviceId: string) => void;
  disconnectFromDevice: () => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = (props) => {
  const {
    onBack,
    artist,
    tracks,
    albums,
    onPlayArtistTracks,
    onPlayNext,
    onCreatePlaylist,
    recentPlaylists,
    onAddTrackToPlaylist,
  } = props;
  const theme = useTheme();
  return (
    <Container>
      <Sidebar active="artists" {...props} />
      <Content>
        <ControlBar {...props} />
        <MainContent displayHeader={false}>
          <Scrollable>
            <BackButton onClick={onBack}>
              <div style={{ marginTop: 2 }}>
                <ArrowBack color={theme.colors.text} />
              </div>
            </BackButton>
            <Artist>{artist.name}</Artist>
            <Buttons>
              <Button
                onClick={() => onPlayArtistTracks(artist.id, false)}
                kind="primary"
              >
                <Label>
                  <Icon>
                    <Play small color="#fff" />
                  </Icon>
                  <div style={{ marginLeft: 7 }}>Play</div>
                </Label>
              </Button>
              <Separator />
              <Button
                onClick={() => onPlayArtistTracks(artist.id, true)}
                kind="secondary"
              >
                <Label>
                  <Shuffle color="#ab28fc" />
                  <div style={{ marginLeft: 7 }}>Shuffle</div>
                </Label>
              </Button>
            </Buttons>
            <Tracks>
              <TracksTable
                tracks={tracks}
                title={
                  <Row>
                    <Title>Tracks</Title>
                    <SeeMore>See all</SeeMore>
                  </Row>
                }
                maxHeight={"initial"}
                onPlayTrack={(id, position) =>
                  onPlayArtistTracks(id, false, position)
                }
                onPlayNext={onPlayNext}
                onCreatePlaylist={onCreatePlaylist}
                recentPlaylists={recentPlaylists}
                onAddTrackToPlaylist={onAddTrackToPlaylist}
              />
            </Tracks>
            <Row>
              <Title>Albums</Title>
              <SeeMore>See all</SeeMore>
            </Row>
            <Grid gridColumns={[3, 4, 5]} gridMargins={[8, 16, 18]}>
              {albums.map((item) => (
                <Cell key={item.id}>
                  <Link to={`/albums/${item.id}`}>
                    {item.cover && <AlbumCover src={item.cover} />}
                    {!item.cover && (
                      <NoAlbumCover>
                        <AlbumIcon size={120} />
                      </NoAlbumCover>
                    )}
                  </Link>
                  <Link to={`/albums/${item.id}`}>
                    <AlbumTitle>{item.title}</AlbumTitle>
                  </Link>
                  <AlbumArtist>{item.artist}</AlbumArtist>
                </Cell>
              ))}
            </Grid>
          </Scrollable>
        </MainContent>
      </Content>
    </Container>
  );
};

export default ArtistDetails;
