import { Input } from "baseui/input";
import { Textarea } from "baseui/textarea";
import {
  Modal,
  ModalHeader,
  ModalBody,
  ModalFooter,
  ModalButton,
} from "baseui/modal";
import { FC } from "react";

export type NewFolderModalProps = {
  isOpen: boolean;
  onClose: () => void;
};

const NewPlaylistModal: FC<NewFolderModalProps> = ({ onClose, isOpen }) => {
  return (
    <Modal onClose={onClose} isOpen={isOpen}>
      <ModalHeader>Create new playlist</ModalHeader>
      <ModalBody>
        <Input
          placeholder="Give your playlist a title"
          overrides={{
            Root: {
              style: ({ $isFocused }) => ({
                borderTopWidth: "0px !important",
                borderLeftWidth: "0px !important",
                borderRightWidth: "0px !important",
                borderBottomWidth: "1px !important",
                borderBottomLeftRadius: "0px !important",
                borderBottomRightRadius: "0px !important",
                borderBottomColor: $isFocused
                  ? "rgb(171, 40, 252)"
                  : "rgba(118, 118, 118, 0.189)",
                marginBottom: "15px",
              }),
            },
            Input: {
              style: {
                backgroundColor: "#fff",
                fontSize: "14px",
                paddingLeft: "0px !important",
                paddingRight: "0px !important",
              },
            },
            InputContainer: {
              style: {
                backgroundColor: "#fff",
              },
            },
          }}
        />
        <Textarea
          placeholder="Write a description"
          overrides={{
            Root: {
              style: ({ $isFocused }) => ({
                borderTopWidth: "0px !important",
                borderLeftWidth: "0px !important",
                borderRightWidth: "0px !important",
                borderBottomWidth: "1px !important",
                borderBottomLeftRadius: "0px !important",
                borderBottomRightRadius: "0px !important",
                borderBottomColor: $isFocused
                  ? "rgb(171, 40, 252)"
                  : "rgba(118, 118, 118, 0.189)",
              }),
            },
            Input: {
              style: {
                backgroundColor: "#fff",
                fontSize: "14px",
                paddingLeft: "0px !important",
                paddingRight: "0px !important",
              },
            },
            InputContainer: {
              style: {
                backgroundColor: "#fff",
              },
            },
          }}
        />
      </ModalBody>
      <ModalFooter>
        <ModalButton onClick={onClose}>Create New</ModalButton>
      </ModalFooter>
    </Modal>
  );
};

export default NewPlaylistModal;
