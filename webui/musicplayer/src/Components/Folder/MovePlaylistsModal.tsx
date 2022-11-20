import { FC } from "react";

export type ModalPlaylistsModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onMovePlaylists: (playlistIds: string[], folderId: string) => void;
  playlists: any[];
};

const MovePlaylistsModal: FC = () => {
  return <div></div>;
};

export default MovePlaylistsModal;
