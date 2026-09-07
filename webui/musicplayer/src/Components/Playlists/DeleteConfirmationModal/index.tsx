import { FC } from "react";
import { Button, Dialog, Icons } from "../../UI";

export type DeleteConfirmationModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onDelete: () => void;
  title: string;
  message: string;
};

/**
 * Destructive confirmation. Not dismissable on a backdrop click — a stray
 * click outside a delete dialog should not be the thing that decides it.
 */
const DeleteConfirmationModal: FC<DeleteConfirmationModalProps> = ({
  isOpen,
  onClose,
  onDelete,
  title,
  message,
}) => (
  <Dialog
    isOpen={isOpen}
    onClose={onClose}
    title={title}
    icon={Icons.trash}
    width={420}
    isDismissable={false}
    footer={
      <>
        <Button variant="ghost" onClick={onClose}>
          Cancel
        </Button>
        <Button
          variant="danger"
          onClick={() => {
            onDelete();
            onClose();
          }}
        >
          Delete
        </Button>
      </>
    }
  >
    <p className="pb-2 text-[13px] text-dim">{message}</p>
  </Dialog>
);

export default DeleteConfirmationModal;
