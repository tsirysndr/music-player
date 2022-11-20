import styled from "@emotion/styled";
import { KIND } from "baseui/button";
import {
  Modal,
  ModalBody,
  ModalFooter as DefaultModalFooter,
  ModalHeader,
} from "baseui/modal";
import { FC } from "react";
import Button from "../../Button";

const Separator = styled.div`
  width: 10px;
`;

const ModalFooter = styled(DefaultModalFooter)`
  display: flex;
  flex-direction: row;
  justify-content: flex-end;
`;

export type DeleteConfirmationModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onDelete: () => void;
  title: string;
  message: string;
};

const DeleteConfirmationModal: FC<DeleteConfirmationModalProps> = (props) => {
  const { isOpen, onClose, onDelete, title, message } = props;
  const _onClose = () => {
    onClose();
  };
  return (
    <Modal onClose={_onClose} isOpen={isOpen}>
      <ModalHeader>{title}</ModalHeader>
      <ModalBody>{message}</ModalBody>
      <ModalFooter>
        <Button kind={KIND.tertiary} onClick={onDelete}>
          Delete
        </Button>
        <Separator />
        <Button kind={KIND.secondary} onClick={_onClose}>
          Cancel
        </Button>
      </ModalFooter>
    </Modal>
  );
};

export default DeleteConfirmationModal;
