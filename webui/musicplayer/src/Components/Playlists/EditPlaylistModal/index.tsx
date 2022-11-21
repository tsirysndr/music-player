import { Input } from "baseui/input";
import { Textarea } from "baseui/textarea";
import {
  Modal,
  ModalHeader,
  ModalBody,
  ModalFooter,
  ModalButton,
} from "baseui/modal";
import { FC, useEffect } from "react";
import { Controller, useForm } from "react-hook-form";

export type EditPlaylistModalProps = {
  playlist?: any;
  isOpen: boolean;
  onClose: () => void;
  onEditPlaylist: (id: string, name: string, description?: string) => void;
};

const EditPlaylistModal: FC<EditPlaylistModalProps> = ({
  playlist,
  onClose,
  isOpen,
  onEditPlaylist,
}) => {
  const { control, handleSubmit, reset } = useForm({
    defaultValues: {
      name: playlist?.name,
      description: playlist?.description,
    },
  });
  const _onEditPlaylist = (data: any) => {
    onEditPlaylist(playlist.id, data.name, data.description);
    onClose();
    reset();
  };
  const _onClose = () => {
    onClose();
    reset();
  };
  useEffect(() => {
    reset({
      name: playlist?.name,
      description: playlist?.description,
    });
  }, [playlist, reset]);
  return (
    <Modal onClose={_onClose} isOpen={isOpen}>
      <ModalHeader>Edit playlist</ModalHeader>
      <ModalBody>
        <Controller
          control={control}
          name="name"
          rules={{ required: true }}
          render={({ field: { onChange, onBlur, value } }) => (
            <Input
              value={value}
              onBlur={onBlur}
              onChange={onChange}
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
          )}
        />
        <Controller
          control={control}
          name="description"
          render={({ field: { onChange, onBlur, value } }) => (
            <Textarea
              value={value}
              onBlur={onBlur}
              onChange={onChange}
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
          )}
        />
      </ModalBody>
      <ModalFooter>
        <ModalButton onClick={handleSubmit(_onEditPlaylist)}>Save</ModalButton>
      </ModalFooter>
    </Modal>
  );
};

EditPlaylistModal.defaultProps = {
  playlist: {},
};

export default EditPlaylistModal;
