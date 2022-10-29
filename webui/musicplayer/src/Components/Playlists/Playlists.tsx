import styled from "@emotion/styled";
import { StatefulMenu } from "baseui/menu";
import { StatefulPopover } from "baseui/popover";
import { FC, useState } from "react";
import AddAlt from "../Icons/AddAlt";
import NewFolderModal from "./NewFolderModal";
import NewPlaylistModal from "./NewPlaylistModal";

const Title = styled.div`
  font-size: 16px;
  font-family: RockfordSansBold;
  margin-bottom: 10px;
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
`;

const Plus = styled.div`
  width: 28px;
`;

export type PlaylistProps = {
  playlists?: any[];
};

const Playlists: FC<PlaylistProps> = ({ playlists }) => {
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
              backgroundColor: "#fff",
            },
          },
        }}
      >
        <Create>
          <Plus>
            <AddAlt />
          </Plus>
          <span>Create ...</span>
        </Create>
      </StatefulPopover>
      <NewFolderModal
        isOpen={newFolderModalOpen}
        onClose={() => setNewFolderModalOpen(false)}
      />
      <NewPlaylistModal
        isOpen={newPlaylistModalOpen}
        onClose={() => setNewPlaylistModalOpen(false)}
      />
      {playlists?.slice(0, 5).map((playlist) => (
        <Item key={playlist.id}>{playlist.name}</Item>
      ))}
    </Container>
  );
};

Playlists.defaultProps = {
  playlists: [],
};

export default Playlists;
