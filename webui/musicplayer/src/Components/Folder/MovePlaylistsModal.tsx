import { Modal, ModalHeader, ModalBody, ModalFooter } from "baseui/modal";
import { FC, useState } from "react";
import Button from "../Button";
import Search from "../Search";
import { ListItem, ListItemLabel } from "baseui/list";
import { Checkbox } from "baseui/checkbox";

export type PlaylistItemProps = {
  playlist: {
    id: string;
    name: string;
  };
  onSelected: (id: string) => void;
  onDeselected: (id: string) => void;
};

const PlaylistItem: FC<PlaylistItemProps> = ({
  playlist,
  onSelected,
  onDeselected,
}) => {
  const [checked, setChecked] = useState(false);
  const { name } = playlist;
  return (
    <ListItem
      endEnhancer={(props) => (
        <Checkbox
          checked={checked}
          onChange={(e) => {
            setChecked(e.target.checked);
            if (e.target.checked) {
              onSelected(playlist.id);
              return;
            }
            onDeselected(playlist.id);
          }}
        ></Checkbox>
      )}
      overrides={{
        Content: {
          style: () => ({
            borderBottomWidth: "0px !important",
          }),
        },
      }}
    >
      <ListItemLabel>{name}</ListItemLabel>
    </ListItem>
  );
};

export type MovePlaylistsModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onMovePlaylists: (playlistIds: string[], folderId: string) => void;
  playlists: any[];
  folderId: string;
};

const MovePlaylistsModal: FC<MovePlaylistsModalProps> = ({
  isOpen,
  onClose,
  onMovePlaylists,
  playlists,
  folderId,
}) => {
  const [selectedPlaylists, setSelectedPlaylists] = useState<string[]>([]);
  const _onMovePlaylists = () => {
    onMovePlaylists(selectedPlaylists, folderId);
    onClose();
  };
  const onSelected = (id: string) => {
    setSelectedPlaylists([...selectedPlaylists, id]);
  };
  const onDeselected = (id: string) => {
    setSelectedPlaylists(selectedPlaylists.filter((p) => p !== id));
  };
  return (
    <Modal onClose={onClose} isOpen={isOpen}>
      <ModalHeader>Move Playlists</ModalHeader>
      <ModalBody>
        <Search onSearch={(q) => {}} height="40px" width="100%" />
        {playlists.map((item: any) => (
          <PlaylistItem
            key={item.id}
            playlist={item}
            onSelected={onSelected}
            onDeselected={onDeselected}
          />
        ))}
      </ModalBody>
      <ModalFooter>
        <Button onClick={_onMovePlaylists}>Move Playlists</Button>
      </ModalFooter>
    </Modal>
  );
};

export default MovePlaylistsModal;
