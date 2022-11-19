import styled from "@emotion/styled";
import { FC } from "react";
import Library from "../Library";
import Playlists from "../Playlists";
import Search from "../Search";

const Container = styled.div`
  height: calc(100vh - 30px);
  padding-top: 30px;
  padding-left: 30px;
  padding-right: 20px;
  min-width: 222px;
  overflow-y: auto;
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
};

const Sidebar: FC<SidebarProps> = (props) => {
  return (
    <Container>
      <Search {...props} />
      <Library {...props} />
      <Playlists {...props} />
    </Container>
  );
};

Sidebar.defaultProps = {
  active: "tracks",
};

export default Sidebar;
