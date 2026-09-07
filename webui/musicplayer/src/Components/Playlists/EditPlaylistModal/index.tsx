import { zodResolver } from "@hookform/resolvers/zod";
import { FC, useEffect } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button, Dialog, Icons, TextAreaField, TextField } from "../../UI";

const schema = z.object({
  name: z.string().trim().min(1, "Give your playlist a name"),
  description: z.string().optional(),
});

type FormValues = z.infer<typeof schema>;

export type EditPlaylistModalProps = {
  playlist?: { id: string; name: string; description?: string | null };
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
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: { name: "", description: "" },
  });

  // The modal is mounted before a playlist is chosen, so the defaults have to
  // be pushed in when one arrives rather than read once at mount.
  useEffect(() => {
    reset({
      name: playlist?.name ?? "",
      description: playlist?.description ?? "",
    });
  }, [playlist, reset]);

  const close = () => {
    onClose();
    reset();
  };

  const submit = (values: FormValues) => {
    if (!playlist) return;
    onEditPlaylist(playlist.id, values.name.trim(), values.description);
    close();
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title="Edit playlist"
      icon={Icons.pencil}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button onClick={handleSubmit(submit)}>Save</Button>
        </>
      }
    >
      <form
        onSubmit={handleSubmit(submit)}
        className="flex flex-col gap-4 pb-1"
      >
        <TextField
          label="NAME"
          autoFocus
          placeholder="Give your playlist a title"
          error={errors.name?.message}
          {...register("name")}
        />
        <TextAreaField
          label="DESCRIPTION"
          placeholder="Write a description"
          {...register("description")}
        />
      </form>
    </Dialog>
  );
};

export default EditPlaylistModal;
