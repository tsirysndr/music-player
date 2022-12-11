import { ListItem, ListItemLabel } from "baseui/list";
import { Modal, ModalHeader, ModalBody } from "baseui/modal";
import { FC } from "react";
import { MusicPlayer } from "@styled-icons/bootstrap";
import { Kodi } from "@styled-icons/simple-icons";
import styled from "@emotion/styled";
import { css } from "@emotion/react";

const List = styled.ul`
  padding: 0;
`;

const Icon = styled.div`
  height: 40px;
  width: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #28fcbc20;
  ${(props) =>
    props.color &&
    css`
      background-color: ${props.color};
    `}
`;

export type ArtworkProps = {
  icon?: "kodi" | "musicplayer";
  color?: string;
};

const Artwork: FC<ArtworkProps> = ({ icon, color }) => {
  return (
    <Icon color={color}>
      {icon !== "kodi" && <MusicPlayer size={20} color="#28fce3" />}
      {icon === "kodi" && <Kodi size={20} color="#28cbfc" />}
    </Icon>
  );
};

Artwork.defaultProps = {
  icon: "musicplayer",
};

export type ConnectModalProps = {
  isOpen: boolean;
  onClose: () => void;
};

const ConnectModal: FC<ConnectModalProps> = ({ onClose, isOpen }) => {
  return (
    <Modal onClose={onClose} isOpen={isOpen}>
      <ModalHeader>Connect to</ModalHeader>
      <ModalBody>
        <List className="connect-list">
          <ListItem
            artwork={() => <Artwork />}
            overrides={{
              Content: {
                style: {
                  borderBottom: "none",
                },
              },
            }}
          >
            <ListItemLabel children={"MacbookPro 1"} />
          </ListItem>
          <ListItem
            artwork={() => <Artwork icon="kodi" color="#28cbfc17" />}
            overrides={{
              Content: {
                style: {
                  borderBottom: "none",
                },
              },
            }}
          >
            <ListItemLabel children={"MacbookPro 2"} />
          </ListItem>
          <ListItem
            artwork={() => <Artwork />}
            overrides={{
              Content: {
                style: {
                  borderBottom: "none",
                },
              },
            }}
          >
            <ListItemLabel children={"MacbookPro 3"} />
          </ListItem>
        </List>
      </ModalBody>
    </Modal>
  );
};

export default ConnectModal;
