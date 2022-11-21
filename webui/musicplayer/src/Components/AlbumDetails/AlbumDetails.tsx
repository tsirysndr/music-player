import styled from "@emotion/styled";
import { FC } from "react";
import { useCover } from "../../Hooks/useCover";
import Button from "../Button";
import ControlBar from "../ControlBar";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import TracksTable from "../TracksTable";
import AlbumIcon from "../Icons/AlbumCover";
import _ from "lodash";
import { Track } from "../../Types";

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
  margin-left: 26px;
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

const NoAlbumCover = styled.div`
  height: 240px;
  width: 240px;
  border-radius: 3px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #ddaefb14;
`;

const Album = styled.div`
  display: flex;
  flex-direction: row;
  margin-top: 90px;
  margin-left: 16px;
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
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  nextTracks: Track[];
  previousTracks: Track[];
  onPlayAlbum: (id: string, shuffle: boolean, position?: number) => void;
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
};

const AlbumDetails: FC<AlbumDetailsProps> = (props) => {
  const {
    onBack,
    album,
    nowPlaying,
    onPlayAlbum,
    onPlayNext,
    onCreatePlaylist,
    onAddTrackToPlaylist,
    recentPlaylists,
  } = props;
  const coverUrl = _.startsWith(album.cover, "https://")
    ? album.cover
    : `/covers/${album.cover}`;
  const { cover } = useCover(coverUrl);
  return (
    <Container>
      <Sidebar active="albums" {...props} />
      <Content>
        <ControlBar {...props} />
        <MainContent displayHeader={false}>
          <Scrollable>
            <BackButton onClick={onBack}>
              <div style={{ marginTop: 2 }}>
                <ArrowBack />
              </div>
            </BackButton>
            <Album>
              {cover && <AlbumCover src={cover} />}
              {!cover && (
                <NoAlbumCover>
                  <AlbumIcon />
                </NoAlbumCover>
              )}
              <Metadata>
                <MetadataContainer>
                  <Title>{album.title}</Title>
                  <Artist>{album.artist}</Artist>
                  <Tracks>{album.tracks.length} TRACKS</Tracks>
                </MetadataContainer>
                <Buttons>
                  <Button
                    onClick={() => onPlayAlbum(album.id, false)}
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
                    onClick={() => onPlayAlbum(album.id, true)}
                    kind="secondary"
                  >
                    <Label>
                      <Shuffle color="#ab28fc" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </Label>
                  </Button>
                </Buttons>
              </Metadata>
            </Album>
            <TracksTable
              tracks={album.tracks.map((track: any) => ({ ...track, cover }))}
              currentTrackId={nowPlaying.id}
              isPlaying={nowPlaying.isPlaying}
              header={["#", "Title", "Artist", "Time"]}
              maxHeight={"initial"}
              onPlayTrack={(id, position) => onPlayAlbum(id, false, position)}
              onPlayNext={onPlayNext}
              onCreatePlaylist={onCreatePlaylist}
              recentPlaylists={recentPlaylists}
              onAddTrackToPlaylist={onAddTrackToPlaylist}
            />
          </Scrollable>
        </MainContent>
      </Content>
    </Container>
  );
};

export default AlbumDetails;
