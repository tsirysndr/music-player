import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC, useState } from "react";
import { Link } from "react-router-dom";
import Button from "../Button";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import PlaylistIcon from "../Icons/PlaylistAlt";
import { Folder as FolderIcon } from "@styled-icons/bootstrap";
import MovePlaylistsModal from "./MovePlaylistsModal";
import { Device } from "../../Types/Device";
import ListeningOn from "../ListeningOn";

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

const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;
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
  onPlayNext: (id: string) => void;
  playlists: any[];
  mainPlaylists: any[];
  onCreatePlaylist: (name: string, description?: string) => void;
  onMovePlaylists: (playlistIds: string[], folderId: string) => void;
  folder?: any;
  currentCastDevice?: Device;
  onFilter: (value: string) => void;
};

const Folder: FC<FolderProps> = (props) => {
  const {
    playlists,
    mainPlaylists,
    folder,
    onMovePlaylists,
    currentCastDevice,
    onFilter,
  } = props;
  const [isMovePlaylistsModalOpen, setIsMovePlaylistsModalOpen] =
    useState(false);
  return (
    <>
      {currentCastDevice && <ListeningOn deviceName={currentCastDevice.name} />}
      <Container>
        <Sidebar active="artists" playlists={mainPlaylists} />
        <Content>
          <ControlBar />
          <MainContent displayHeader={false}>
            <Scrollable>
              {folder?.playlists?.length > 0 && (
                <Scrollable>
                  <MainContent
                    title="Playlists"
                    placeholder="Filter Playlists"
                    onFilter={onFilter}
                  >
                    <Wrapper>
                      <Grid
                        gridColumns={[2, 3, 4, 6]}
                        gridMargins={[8, 16, 18]}
                      >
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
    </>
  );
};

export default Folder;
