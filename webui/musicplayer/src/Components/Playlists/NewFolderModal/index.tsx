import { zodResolver } from "@hookform/resolvers/zod";
import { FC } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button, Dialog, Icons, TextField } from "../../UI";

const schema = z.object({
  name: z.string().trim().min(1, "Give your folder a name"),
});

type FormValues = z.infer<typeof schema>;

export type NewFolderModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onCreateFolder: (name: string) => void;
};

const NewFolderModal: FC<NewFolderModalProps> = ({
  onClose,
  isOpen,
  onCreateFolder,
}) => {
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: { name: "" },
  });

  const close = () => {
    onClose();
    reset();
  };

  const submit = (values: FormValues) => {
    onCreateFolder(values.name.trim());
    close();
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title="Create folder"
      icon={Icons.folder}
      width={420}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button onClick={handleSubmit(submit)}>Create folder</Button>
        </>
      }
    >
      <form onSubmit={handleSubmit(submit)} className="pb-1">
        <TextField
          label="NAME"
          autoFocus
          placeholder="Give your folder a name"
          error={errors.name?.message}
          {...register("name")}
        />
      </form>
    </Dialog>
  );
};

export default NewFolderModal;
