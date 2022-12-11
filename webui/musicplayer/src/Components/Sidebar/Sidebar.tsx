import styled from "@emotion/styled";
import { FC, useState } from "react";
import Library from "../Library";
import Playlists from "../Playlists";
import Search from "../Search";
import { PlugConnected } from "@styled-icons/fluentui-system-regular";
import ConnectModal from "./ConnectModal";

const Container = styled.div`
  height: calc(100vh - 30px);
  padding-top: 30px;
  padding-left: 30px;
  padding-right: 20px;
  min-width: 222px;
  overflow-y: auto;
`;

const ConnectButton = styled.button`
  border: none;
  background-color: #ab28fc0d;
  height: 32px;
  width: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: absolute;
  bottom: 0;
  left: 0;
`;

export type SidebarProps = {
  active?: string;
  onClickLibraryItem: (item: string) => void;
  onSearch: (query: string) => void;
  folders: any[];
  playlists: any[];
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

const Sidebar: FC<SidebarProps> = (props) => {
  const [openConnectModal, setOpenConnectModal] = useState(false);
  return (
    <Container>
      <Search {...props} />
      <Library {...props} />
      <Playlists {...props} />
      <ConnectButton onClick={() => setOpenConnectModal(true)}>
        <PlugConnected size={20} color="#ab28fc" />
      </ConnectButton>
      <ConnectModal
        isOpen={openConnectModal}
        onClose={() => setOpenConnectModal(false)}
      />
    </Container>
  );
};

Sidebar.defaultProps = {
  active: "tracks",
};

export default Sidebar;
