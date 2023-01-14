import styled from "@emotion/styled";
import { FC, useContext, useState } from "react";
import Library from "../Library";
import Playlists from "../Playlists";
import Search from "../Search";
import { PlugConnected } from "@styled-icons/fluentui-system-regular";
import ConnectModal from "./ConnectModal";
import { Device } from "../../Types/Device";
import { Sun, MoonStars } from "@styled-icons/bootstrap";
import { useTheme } from "@emotion/react";
import { ThemeContext } from "../../Providers/ThemeProvider";

const Button = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  &:hover {
    opacity: 0.6;
  }
`;

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
  background-color: ${(props) => props.theme.colors.secondaryBackground};
  height: 32px;
  ${(props) => (props.connected ? "width: 272px;" : "40px")}
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: absolute;
  bottom: 0;
  left: 0;
`;

const ConnectText = styled.span`
  font-size: 13.5px;
  color: #ab28fc;
  flex: 1;
  font-family: RockfordSansRegular;
  text-overflow: ellipsis;
  margin-left: 10px;
  margin-right: 10px;
  overflow: hidden;
  white-space: nowrap;
`;

const DarkModeButtonWrapper = styled.div`
  position: absolute;
  bottom: 50px;
`;

const DarkModeButton = () => {
  const theme = useTheme();
  const { theme: currentTheme, setTheme } = useContext(ThemeContext);
  return (
    <DarkModeButtonWrapper>
      <Button
        onClick={() => setTheme(currentTheme === "dark" ? "light" : "dark")}
      >
        {currentTheme === "light" && (
          <Sun size={18} color={theme.colors.text} />
        )}
        {currentTheme === "dark" && (
          <MoonStars size={18} color={theme.colors.text} />
        )}
      </Button>
    </DarkModeButtonWrapper>
  );
};

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
  currentDevice?: Device;
  connectToDevice: (deviceId: string) => void;
  disconnectFromDevice: () => void;
};

const Sidebar: FC<SidebarProps> = (props) => {
  const [openConnectModal, setOpenConnectModal] = useState(false);
  const { currentDevice } = props;
  const connected = !!currentDevice;
  return (
    <Container>
      <Search {...props} />
      <Library {...props} />
      <Playlists {...props} />

      <DarkModeButton />

      <ConnectButton
        onClick={() => setOpenConnectModal(true)}
        connected={connected}
      >
        <PlugConnected size={20} color="#ab28fc" />
        {connected && <ConnectText>Device: {currentDevice.name}</ConnectText>}
      </ConnectButton>
      <ConnectModal
        isOpen={openConnectModal}
        onClose={() => setOpenConnectModal(false)}
        devices={props.devices}
        currentDevice={props.currentDevice}
        connectToDevice={props.connectToDevice}
        disconnectFromDevice={props.disconnectFromDevice}
      />
    </Container>
  );
};

Sidebar.defaultProps = {
  active: "tracks",
  devices: [],
};

export default Sidebar;
