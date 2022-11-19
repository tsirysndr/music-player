import { Menu } from "baseui/menu";
import { FC, useState } from "react";
import { PlayList as PlaylistIcon } from "@styled-icons/remix-fill";
import styled from "@emotion/styled";
import EditPlaylistModal from "../EditPlaylistModal";
import DeleteConfirmationModal from "../DeleteConfirmationModal";

const IconWrapper = styled.div`
  height: 40px;
  width: 40px;
  background-color: #fbf5fe;
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 3px;
`;

const Header = styled.div`
  height: 58px;
  display: flex;
  flex-direction: row;
  align-items: center;
  padding-left: 5px;
  padding-right: 5px;
  border-bottom: 1px solid #e0e0e0cc;
`;

const Title = styled.div`
  font-size: 15px;
`;

const PlaylistDetails = styled.div`
  margin-left: 12px;
`;

export type ContextMenuProps = {
  playlist: any;
  close: () => void;
  handlers: any;
};

type Label =
  | "Play Now"
  | "Shuffle"
  | "Play Next"
  | "Add to Playlist"
  | "Move to Folder"
  | "Edit Playlist"
  | "Delete Playlist";

const ContextMenu: FC<ContextMenuProps> = ({ close, playlist, handlers }) => {
  return (
    <div style={{ width: 205 }}>
      <Header>
        <IconWrapper>
          <PlaylistIcon width={18} height={18} color="#ab28fc" />
        </IconWrapper>
        <PlaylistDetails>
          <Title>{playlist.name}</Title>
        </PlaylistDetails>
      </Header>
      <Menu
        overrides={{
          List: {
            style: {
              boxShadow: "none",
            },
          },
          ListItem: {
            props: {
              onClick: (x: any) => {
                handlers[x.target.innerText as Label]();
              },
            },
            style: () => ({
              ":hover": {
                backgroundColor: "#f7f7f8",
              },
            }),
          },
        }}
        items={[
          {
            id: "1",
            label: "Play Now",
          },
          {
            id: "2",
            label: "Shuffle",
          },
          {
            id: "3",
            label: "Play Next",
          },
          {
            id: "4",
            label: "Add to Playlist",
          },
          {
            id: "5",
            label: "Move to Folder",
          },
          {
            id: "6",
            label: "Edit Playlist",
          },
          {
            id: "7",
            label: "Delete Playlist",
          },
        ]}
      />
    </div>
  );
};

export default ContextMenu;
