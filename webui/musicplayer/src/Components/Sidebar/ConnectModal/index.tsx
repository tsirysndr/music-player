import { ListItem, ListItemLabel } from "baseui/list";
import { Modal, ModalHeader, ModalBody } from "baseui/modal";
import { FC } from "react";
import { MusicPlayer } from "@styled-icons/bootstrap";
import { Kodi } from "@styled-icons/simple-icons";
import styled from "@emotion/styled";
import { css, useTheme } from "@emotion/react";
import { Device } from "../../../Types/Device";
import { Button, SHAPE, SIZE } from "baseui/button";

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

const Placeholder = styled.div`
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
`;

export type ArtworkProps = {
  icon?: string;
  color?: string;
};

const Artwork: FC<ArtworkProps> = ({ icon, color }) => {
  return (
    <Icon color={color}>
      {icon !== "xbmc" && <MusicPlayer size={20} color="#28fce3" />}
      {icon === "xbmc" && <Kodi size={20} color="#28cbfc" />}
    </Icon>
  );
};

Artwork.defaultProps = {
  icon: "music-player",
};

export type ConnectModalProps = {
  isOpen: boolean;
  onClose: () => void;
  devices: Device[];
  currentDevice?: Device;
  connectToDevice: (id: string) => void;
  disconnectFromDevice: () => void;
};

const ConnectModal: FC<ConnectModalProps> = ({
  onClose,
  isOpen,
  devices,
  currentDevice,
  connectToDevice,
  disconnectFromDevice,
}) => {
  const theme = useTheme();
  const _connectToDevice = (id: string) => {
    if (currentDevice?.id === id) {
      return;
    }
    connectToDevice(id);
    onClose();
  };
  const _disconnectFromDevice = () => {
    disconnectFromDevice();
    onClose();
  };
  return (
    <Modal onClose={onClose} isOpen={isOpen}>
      <ModalHeader>Connect to</ModalHeader>
      <ModalBody>
        <List className="connect-list">
          {devices.map((device) => (
            <div onClick={() => _connectToDevice(device.id)}>
              <ListItem
                artwork={() => (
                  <Artwork
                    icon={device.type}
                    color={device.name === "xbmc" ? "#28cbfc17" : undefined}
                  />
                )}
                overrides={{
                  Root: {
                    style: {
                      ":hover": {
                        backgroundColor: theme.colors.hover,
                      },
                    },
                  },
                  Content: {
                    style: {
                      borderBottom: "none",
                    },
                  },
                }}
                endEnhancer={() => (
                  <>
                    {device.id === currentDevice?.id && (
                      <Button
                        size={SIZE.compact}
                        shape={SHAPE.pill}
                        kind="tertiary"
                        onClick={() => _disconnectFromDevice()}
                      >
                        Disconnect
                      </Button>
                    )}
                  </>
                )}
              >
                <ListItemLabel children={device.name} />
              </ListItem>
            </div>
          ))}
          {devices.length === 0 && (
            <Placeholder>
              No devices found. Start Music Player on other devices to connect.
            </Placeholder>
          )}
        </List>
      </ModalBody>
    </Modal>
  );
};

ConnectModal.defaultProps = {
  devices: [],
};

export default ConnectModal;
