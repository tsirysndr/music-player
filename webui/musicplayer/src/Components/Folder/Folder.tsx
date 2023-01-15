import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC, useState } from "react";
import { Link } from "react-router-dom";
import Button from "../Button";
import ControlBar from "../ControlBar";
import ArrowBack from "../Icons/ArrowBack";
import Play from "../Icons/Play";
import Shuffle from "../Icons/Shuffle";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import PlaylistIcon from "../Icons/PlaylistAlt";
import { Track } from "../../Types";
import { Folder as FolderIcon } from "@styled-icons/bootstrap";
import MovePlaylistsModal from "./MovePlaylistsModal";
import { Device } from "../../Types/Device";

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
`;

const PlaylistName = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  margin-top: 15px;
  color: ${(props) => props.theme.colors.text};
`;

const Row = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  margin-right: 50px;
  margin-left: 10px;
`;

const NoPlaylistCover = styled.div`
  height: 220px;
  width: 220px;
  border-radius: 5px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #ddaefb14;
`;

const PlaceholderWrapper = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  height: calc(100vh - 100px);
  flex-direction: column;
`;

const Placeholder = styled.div`
  font-family: RockfordSansRegular;
  text-align: center;
  color: rgb(45, 44, 44);
  margin-top: 50px;
  margin-bottom: 35px;
`;

const Wrapper = styled.div`
  margin-top: 34px;
  margin-left: 10px;
`;

export type PlaylistProps = {
  playlist: any;
};

const Playlist: FC<PlaylistProps> = ({ playlist }) => {
  return (
    <>
      <Link
        to={`/playlists/${playlist.id}`}
        style={{ textDecoration: "initial" }}
      >
        <NoPlaylistCover>
          <PlaylistIcon
            size={48}
            color="#ab28fc"
            style={{ marginRight: -38, marginTop: 30 }}
          />
        </NoPlaylistCover>
        <PlaylistName>{playlist.name}</PlaylistName>
      </Link>
    </>
  );
};

export type FolderProps = {
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
  mainPlaylists: any[];
  onCreateFolder: (name: string) => void;
  onCreatePlaylist: (name: string, description?: string) => void;
  onDeleteFolder: (id: string) => void;
  onDeletePlaylist: (id: string) => void;
  onEditFolder: (id: string, name: string) => void;
  onEditPlaylist: (id: string, name: string, description?: string) => void;
  onPlayPlaylist: (
    playlistId: string,
    shuffle: boolean,
    position?: number
  ) => void;
  onMovePlaylists: (playlistIds: string[], folderId: string) => void;
  folder?: any;
  devices: Device[];
  castDevices: Device[];
  currentDevice?: Device;
  connectToDevice: (deviceId: string) => void;
  disconnectFromDevice: () => void;
};

const Folder: FC<FolderProps> = (props) => {
  const {
    onPlayNext,
    onCreatePlaylist,
    playlists,
    mainPlaylists,
    folder,
    onMovePlaylists,
  } = props;
  const [isMovePlaylistsModalOpen, setIsMovePlaylistsModalOpen] =
    useState(false);
  return (
    <Container>
      <Sidebar active="artists" {...props} playlists={mainPlaylists} />
      <Content>
        <ControlBar {...props} />
        <MainContent displayHeader={false}>
          <Scrollable>
            {folder?.playlists?.length > 0 && (
              <Scrollable>
                <MainContent title="Playlists" placeholder="Filter Playlists">
                  <Wrapper>
                    <Grid gridColumns={[2, 3, 4, 6]} gridMargins={[8, 16, 18]}>
                      {folder?.playlists.map((item: any) => (
                        <Cell key={item.id}>
                          <Playlist playlist={item} />
                        </Cell>
                      ))}
                    </Grid>
                  </Wrapper>
                </MainContent>
              </Scrollable>
            )}
            {folder?.playlists?.length === 0 && (
              <PlaceholderWrapper>
                <FolderIcon size={100} color="rgb(70, 70, 70)" />
                <Placeholder>
                  Start moving playlists to your folder.
                </Placeholder>
                <Button onClick={() => setIsMovePlaylistsModalOpen(true)}>
                  Move Playlists
                </Button>
              </PlaceholderWrapper>
            )}
          </Scrollable>
        </MainContent>
      </Content>
      <MovePlaylistsModal
        isOpen={isMovePlaylistsModalOpen}
        onClose={() => setIsMovePlaylistsModalOpen(false)}
        onMovePlaylists={onMovePlaylists}
        playlists={playlists}
        folderId={folder?.id}
      />
    </Container>
  );
};

export default Folder;
