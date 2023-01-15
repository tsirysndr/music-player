import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC } from "react";
import { Link } from "react-router-dom";
import Button from "../Button";
import ControlBar from "../ControlBar";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import PlaylistIcon from "../Icons/PlaylistAlt";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import TracksTable from "../TracksTable";
import { Track } from "../../Types";
import { useTimeFormat } from "../../Hooks/useFormat";
import { resourceUriResolver } from "../../ResourceUriResolver";
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
`;

const Buttons = styled.div`
  display: flex;
  flex-direction: row;
  margin-top: 26px;
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

const NoCover = styled.div`
  height: 240px;
  width: 240px;
  border-radius: 5px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #ddaefb14;
`;
const Title = styled.div`
  font-family: RockfordSansBold;
  font-size: 32px;
  color: ${(props) => props.theme.colors.text};
`;
const Placeholder = styled.div`
  font-family: RockfordSansRegular;
  text-align: center;
  color: rgb(45, 44, 44);
  margin-top: 50px;
`;

const PlaylistDetails = styled.div`
  display: flex;
  flex-direction: column;
  margin-left: 26px;
  height: 240px;
`;

const PlaylistDetailsWrapper = styled.div`
  flex: 1;
  display: flex;
  align-items: center;
`;

const Header = styled.div`
  display: flex;
  flex-direction: row;
  margin-top: 90px;
  margin-left: 16px;
`;

export type PlaylistProps = {
  playlist: any;
  onBack: () => void;
  onClickLibraryItem: (item: string) => void;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
  nextTracks: Track[];
  previousTracks: Track[];
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
  onAddTrackToPlaylist: (playlistId: string, trackId: string) => void;
  onPlayPlaylist: (
    playlistId: string,
    shuffle: boolean,
    position?: number
  ) => void;
  recentPlaylists: any[];
  devices: Device[];
  castDevices: Device[];
  currentDevice?: Device;
  connectToDevice: (deviceId: string) => void;
  disconnectFromDevice: () => void;
};

const Playlist: FC<PlaylistProps> = (props) => {
  const {
    onBack,
    onPlayNext,
    onCreatePlaylist,
    onAddTrackToPlaylist,
    onPlayPlaylist,
    nowPlaying,
    playlist,
    recentPlaylists,
  } = props;
  const { formatTime } = useTimeFormat();
  const tracks =
    (playlist?.tracks || []).map((track: any) => ({
      id: track.id,
      title: track.title,
      artist: track.artist,
      album: track.albumTitle,
      time: formatTime(track.duration! * 1000),
      cover: track.cover
        ? resourceUriResolver.resolve(`/covers/${track.cover}`)
        : undefined,
      artistId: track.artistId,
      albumId: track.albumId,
    })) || [];
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
            <Header>
              <NoCover>
                <PlaylistIcon
                  size={48}
                  color="#ab28fc"
                  style={{ marginRight: -38, marginTop: 38 }}
                />
              </NoCover>
              <PlaylistDetails>
                <PlaylistDetailsWrapper>
                  <Title>{playlist.name}</Title>
                </PlaylistDetailsWrapper>
                <Buttons>
                  <Button
                    onClick={() => onPlayPlaylist(playlist.id, false)}
                    kind="primary"
                    disabled={!tracks.length}
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
                    onClick={() => onPlayPlaylist(playlist.id, true)}
                    kind="secondary"
                    disabled={!tracks.length}
                  >
                    <Label>
                      <Shuffle color="#ab28fc" />
                      <div style={{ marginLeft: 7 }}>Shuffle</div>
                    </Label>
                  </Button>
                </Buttons>
              </PlaylistDetails>
            </Header>
            {tracks.length === 0 && (
              <Placeholder>
                Start building your playlist with tracks by tapping on ‘Add to
                playlist’ in the option menu.
              </Placeholder>
            )}
            {tracks.length > 0 && (
              <TracksTable
                tracks={tracks}
                currentTrackId={nowPlaying.id}
                isPlaying={nowPlaying.isPlaying}
                header={["Title", "Artist", "Album", "Time"]}
                maxHeight={"initial"}
                onPlayTrack={(id, position) =>
                  onPlayPlaylist(id, false, position)
                }
                onPlayNext={onPlayNext}
                onCreatePlaylist={onCreatePlaylist}
                recentPlaylists={recentPlaylists}
                onAddTrackToPlaylist={onAddTrackToPlaylist}
              />
            )}
          </Scrollable>
        </MainContent>
      </Content>
    </Container>
  );
};

Playlist.defaultProps = {
  playlist: {
    name: "Playlist",
  },
};

export default Playlist;
