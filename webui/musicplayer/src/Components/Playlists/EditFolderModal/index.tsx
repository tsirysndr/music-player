import { zodResolver } from "@hookform/resolvers/zod";
import { FC, useEffect } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button, Dialog, Icons, TextField } from "../../UI";

const schema = z.object({
  name: z.string().trim().min(1, "Give your folder a name"),
});

type FormValues = z.infer<typeof schema>;

export type EditFolderModalProps = {
  folder?: { id: string; name: string };
  isOpen: boolean;
  onClose: () => void;
  onEditFolder: (id: string, name: string) => void;
};

const EditFolderModal: FC<EditFolderModalProps> = ({
  folder,
  onClose,
  isOpen,
  onEditFolder,
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

  useEffect(() => {
    reset({ name: folder?.name ?? "" });
  }, [folder, reset]);

  const close = () => {
    onClose();
    reset();
  };

  const submit = (values: FormValues) => {
    if (!folder) return;
    onEditFolder(folder.id, values.name.trim());
    close();
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title="Edit folder"
      icon={Icons.folder}
      width={420}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button onClick={handleSubmit(submit)}>Save</Button>
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

export default EditFolderModal;
