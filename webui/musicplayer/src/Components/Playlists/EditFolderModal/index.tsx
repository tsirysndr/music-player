import { Input } from "baseui/input";
import {
  Modal,
  ModalHeader,
  ModalBody,
  ModalFooter,
  ModalButton,
} from "baseui/modal";
import { FC } from "react";
import { Controller, useForm } from "react-hook-form";

export type EditFolderModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onEditFolder: (name: string) => void;
};

const EditFolderModal: FC<EditFolderModalProps> = ({
  onClose,
  isOpen,
  onEditFolder,
}) => {
  const { control, handleSubmit, reset } = useForm();
  const _onEditFolder = (data: any) => {
    onEditFolder(data.name);
    onClose();
    reset();
  };
  const _onClose = () => {
    onClose();
    reset();
  };
  return (
    <Modal onClose={_onClose} isOpen={isOpen}>
      <ModalHeader>Edit Folder</ModalHeader>
      <ModalBody>
        <Controller
          name="name"
          rules={{ required: true }}
          control={control}
          render={({ field: { onChange, onBlur, value } }) => (
            <Input
              value={value}
              onBlur={onBlur}
              onChange={onChange}
              placeholder="Give your folder a name"
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
          )}
        />
      </ModalBody>
      <ModalFooter>
        <ModalButton onClick={handleSubmit(_onEditFolder)}>Save</ModalButton>
      </ModalFooter>
    </Modal>
  );
};

export default EditFolderModal;
