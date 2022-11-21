import styled from "@emotion/styled";
import { StatefulMenu } from "baseui/menu";
import { FC } from "react";
import { Folder as FolderIcon } from "@styled-icons/bootstrap";

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

const FolderDetails = styled.div`
  margin-left: 12px;
`;

const Title = styled.div`
  font-size: 15px;
`;

type Label = "Rename" | "Delete Folder" | "Create Playlist";

export type FolderContextProps = {
  folder: any;
  close: () => void;
  handlers: any;
};

const FolderContextMenu: FC<FolderContextProps> = ({
  close,
  folder,
  handlers,
}) => {
  return (
    <div style={{ width: 205 }}>
      <Header>
        <IconWrapper>
          <FolderIcon width={18} height={18} color="#ab28fc" />
        </IconWrapper>
        <FolderDetails>
          <Title>{folder.name}</Title>
        </FolderDetails>
      </Header>
      <StatefulMenu
        overrides={{
          List: {
            style: {
              boxShadow: "none",
            },
          },
          ListItem: {
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
            label: "Rename",
          },
          {
            id: "2",
            label: "Delete Folder",
          },
          {
            id: "3",
            label: "Create Playlist",
          },
        ]}
        onItemSelect={({ item }) => {
          handlers[item.label as Label]();
          close();
        }}
      />
    </div>
  );
};

export default FolderContextMenu;
