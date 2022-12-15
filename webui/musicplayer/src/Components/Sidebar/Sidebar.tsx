import styled from "@emotion/styled";
import { FC, useState } from "react";
import Library from "../Library";
import Playlists from "../Playlists";
import Search from "../Search";
import { PlugConnected } from "@styled-icons/fluentui-system-regular";
import ConnectModal from "./ConnectModal";
import { Device } from "../../Types/Device";

const Container = styled.div`
  height: calc(100vh - 30px);
  padding-top: 30px;
  padding-left: 30px;
  padding-right: 20px;
  min-width: 222px;
  overflow-y: auto;
`;

const ConnectButton = styled.button<{
  connected?: boolean;
}>`
  border: none;
  background-color: #fef2f8;
  height: 32px;
  ${(props) => (props.connected ? "width: 272px;" : "40px")}
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: absolute;
  bottom: 0;
  left: 0;
  overflow: hidden;
`;

const ConnectText = styled.span`
  font-size: 13.5px;
  color: #eb2f96;
  flex: 1;
  font-family: RockfordSansRegular;
  text-overflow: ellipsis;
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
  devices: Device[];
};

const Sidebar: FC<SidebarProps> = (props) => {
  const [openConnectModal, setOpenConnectModal] = useState(false);
  const connected = false;
  return (
    <Container>
      <Search {...props} />
      <Library {...props} />
      <Playlists {...props} />
      <ConnectButton
        onClick={() => setOpenConnectModal(true)}
        connected={connected}
      >
        <PlugConnected size={20} color="#eb2f96" />
        {connected && <ConnectText>Music Player (Macbook Pro)</ConnectText>}
      </ConnectButton>
      <ConnectModal
        isOpen={openConnectModal}
        onClose={() => setOpenConnectModal(false)}
        devices={props.devices}
      />
    </Container>
  );
};

Sidebar.defaultProps = {
  active: "tracks",
};

export default Sidebar;
