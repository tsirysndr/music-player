import styled from "@emotion/styled";
import { StatefulMenu } from "baseui/menu";
import { StatefulPopover, Popover } from "baseui/popover";
import { FC, useState } from "react";
import AddAlt from "../Icons/AddAlt";
import NewFolderModal from "./NewFolderModal";
import NewPlaylistModal from "./NewPlaylistModal";
import { Folder as FolderIcon } from "@styled-icons/bootstrap";
import { Link, useParams } from "react-router-dom";
import FolderContextMenu from "./FolderContextMenu";
import ContextMenu from "./ContextMenu";
import DeleteConfirmationModal from "./DeleteConfirmationModal";
import EditPlaylistModal from "./EditPlaylistModal";
import EditFolderModal from "./EditFolderModal";
import { useTheme } from "@emotion/react";

const Title = styled.div`
  font-size: 16px;
  font-family: RockfordSansBold;
  margin-bottom: 10px;
  color: ${(props) => props.theme.colors.text};
`;

const Container = styled.div`
  margin-left: 10px;
`;

const Item = styled.div<{ active?: boolean }>`
  cursor: pointer;
  height: 45px;
  width: 183px;
  display: flex;
  align-items: center;
  font-size: 14px;
  color: ${(props) => props.theme.colors.text};
  ${(props) => (props.active ? "color: #ab28fc;" : "initial")}
`;

const Create = styled.div`
  height: 45px;
  width: 183px;
  display: flex;
  flex-direction: row;
  align-items: center;
  font-size: 14px;
  cursor: pointer;
  color: ${(props) => props.theme.colors.text};
`;

const Plus = styled.div`
  width: 28px;
`;

const FolderItem = styled.div`
  display: flex;
  flex-direction: row;
  align-items: center;
  cursor: pointer;
`;

export type FolderProps = {
  id: string;
  folder: any;
  onDeleteFolder: (id: string) => void;
  onEditFolder: (id: string, name: string) => void;
};

const Folder: FC<FolderProps> = ({
  id,
  folder,
  onDeleteFolder,
  onEditFolder,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [editFolderModalIsOpen, setEditFolderModalIsOpen] =
    useState<boolean>(false);
  const [deleteConfirmationModalIsOpen, setDeleteConfirmationModalIsOpen] =
    useState<boolean>(false);
  const [newPlaylistModalIsOpen, setNewPlaylistModalIsOpen] =
    useState<boolean>(false);
  const handlers = {
    Rename: () => setEditFolderModalIsOpen(true),
    "Delete Folder": () => setDeleteConfirmationModalIsOpen(true),
    "Create Playlist": () => setNewPlaylistModalIsOpen(true),
  };
  const theme = useTheme();
  return (
    <>
      <Popover
        key={folder.id}
        placement="rightTop"
        isOpen={isOpen}
        autoFocus={false}
        onClickOutside={() => setIsOpen(false)}
        content={() => (
          <FolderContextMenu
            close={() => setIsOpen(false)}
            folder={folder}
            handlers={handlers}
          />
        )}
        overrides={{
          Inner: {
            style: {
              backgroundColor: theme.colors.popoverBackground,
            },
          },
        }}
      >
        <Link
          to={`/folders/${folder.id}`}
          style={{ textDecoration: "initial" }}
          onContextMenu={(e) => {
            e.preventDefault();
            setIsOpen(true);
          }}
        >
          <FolderItem>
            <FolderIcon
              size={18}
              style={{ marginRight: 10 }}
              color={theme.colors.icon}
            />
            <Item active={folder.id === id}>{folder.name}</Item>
          </FolderItem>
        </Link>
      </Popover>
      <EditFolderModal
        isOpen={editFolderModalIsOpen}
        onClose={() => setEditFolderModalIsOpen(false)}
        onEditFolder={onEditFolder}
        folder={folder}
      />
      <DeleteConfirmationModal
        isOpen={deleteConfirmationModalIsOpen}
        onClose={() => setDeleteConfirmationModalIsOpen(false)}
        onDelete={() => onDeleteFolder(folder.id)}
        title={"Delete Folder"}
        message={
          "This permanently deletes your folder and all playlists in it."
        }
      />
      <NewPlaylistModal
        isOpen={newPlaylistModalIsOpen}
        onClose={() => setNewPlaylistModalIsOpen(false)}
        onCreatePlaylist={() => {}}
      />
    </>
  );
};

export type PlaylistProps = {
  id: string;
  playlist: any;
  onDeletePlaylist: (id: string) => void;
  onEditPlaylist: (id: string, name: string, description?: string) => void;
  onPlayPlaylist: (
    playlistId: string,
    shuffle: boolean,
    position?: number
  ) => void;
};

const Playlist: FC<PlaylistProps> = ({
  playlist,
  id,
  onDeletePlaylist,
  onEditPlaylist,
  onPlayPlaylist,
}) => {
  const theme = useTheme();
  const [isOpen, setIsOpen] = useState(false);
  const [editPlaylistModalIsOpen, setEditPlaylistModalIsOpen] =
    useState<boolean>(false);
  const [deleteConfirmationModalIsOpen, setDeleteConfirmationModalIsOpen] =
    useState<boolean>(false);
  const handlers = {
    "Play Now": (id: string) => onPlayPlaylist(id, false),
    Shuffle: (id: string) => onPlayPlaylist(id, true),
    "Play Next": () => {},
    "Add to Playlist": () => {},
    "Move to Folder": () => {},
    "Edit Playlist": () => setEditPlaylistModalIsOpen(true),
    "Delete Playlist": () => setDeleteConfirmationModalIsOpen(true),
  };
  return (
    <>
      <Popover
        placement="rightTop"
        isOpen={isOpen}
        autoFocus={false}
        onClickOutside={() => setIsOpen(false)}
        content={() => (
          <ContextMenu
            close={() => setIsOpen(false)}
            playlist={playlist}
            handlers={handlers}
          />
        )}
        overrides={{
          Inner: {
            style: {
              backgroundColor: theme.colors.popoverBackground,
            },
          },
        }}
      >
        <Link
          to={`/playlists/${playlist.id}`}
          style={{ textDecoration: "initial" }}
          onContextMenu={(e) => {
            e.preventDefault();
            setIsOpen(true);
          }}
        >
          <Item
            active={playlist.id === id}
            style={
              {
                /*marginLeft: 28*/
              }
            }
          >
            {playlist.name}
          </Item>
        </Link>
      </Popover>
      <EditPlaylistModal
        isOpen={editPlaylistModalIsOpen}
        onClose={() => setEditPlaylistModalIsOpen(false)}
        onEditPlaylist={onEditPlaylist}
        playlist={playlist}
      />
      <DeleteConfirmationModal
        isOpen={deleteConfirmationModalIsOpen}
        onClose={() => setDeleteConfirmationModalIsOpen(false)}
        onDelete={() => onDeletePlaylist(playlist.id)}
        title={"Delete Playlist"}
        message={`Are you sure you want to delete this playlist?

        This action cannot be undone.`}
      />
    </>
  );
};

export type PlaylistsProps = {
  playlists?: any[];
  folders?: any[];
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
};

const Playlists: FC<PlaylistsProps> = ({
  folders,
  playlists,
  onCreateFolder,
  onCreatePlaylist,
  onDeleteFolder,
  onDeletePlaylist,
  onEditFolder,
  onEditPlaylist,
  onPlayPlaylist,
}) => {
  const theme = useTheme();
  const { id } = useParams();
  const [newFolderModalOpen, setNewFolderModalOpen] = useState(false);
  const [newPlaylistModalOpen, setNewPlaylistModalOpen] = useState(false);
  return (
    <Container>
      <Title>Playlists</Title>
      <StatefulPopover
        placement="bottom"
        autoFocus={false}
        dismissOnClickOutside={true}
        dismissOnEsc={true}
        content={({ close }) => (
          <div style={{ width: 205 }}>
            <StatefulMenu
              overrides={{
                List: {
                  style: {
                    boxShadow: "none",
                    backgroundColor: theme.colors.popoverBackground,
                  },
                },
              }}
              items={[
                {
                  label: "Folder",
                },
                {
                  label: "Playlist",
                },
              ]}
              onItemSelect={({ item }) => {
                close();
                if (item.label === "Folder") {
                  setNewFolderModalOpen(true);
                } else if (item.label === "Playlist") {
                  setNewPlaylistModalOpen(true);
                }
              }}
            />
          </div>
        )}
        overrides={{
          Inner: {
            style: {
              backgroundColor: theme.colors.popoverBackground,
            },
          },
        }}
      >
        <Create>
          <Plus>
            <AddAlt color={theme.colors.text} />
          </Plus>
          <span>Create ...</span>
        </Create>
      </StatefulPopover>
      <NewFolderModal
        isOpen={newFolderModalOpen}
        onClose={() => setNewFolderModalOpen(false)}
        onCreateFolder={onCreateFolder}
      />
      <NewPlaylistModal
        isOpen={newPlaylistModalOpen}
        onClose={() => setNewPlaylistModalOpen(false)}
        onCreatePlaylist={onCreatePlaylist}
      />
      {folders?.map((folder) => (
        <Folder
          key={folder.id}
          id={id!}
          folder={folder}
          onDeleteFolder={onDeleteFolder}
          onEditFolder={onEditFolder}
        />
      ))}
      {playlists?.map((playlist) => (
        <Playlist
          key={playlist.id}
          id={id!}
          playlist={playlist}
          onDeletePlaylist={onDeletePlaylist}
          onEditPlaylist={onEditPlaylist}
          onPlayPlaylist={onPlayPlaylist}
        />
      ))}
    </Container>
  );
};

Playlists.defaultProps = {
  playlists: [],
  folders: [],
};

export default Playlists;
