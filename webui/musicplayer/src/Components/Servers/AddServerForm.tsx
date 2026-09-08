import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { Button, Dialog, Icons, Select, TextField } from "../UI";
import type { SourceKind } from "./Servers";

const schema = z.object({
  kind: z.string().min(1, "Pick a kind of server"),
  name: z.string().trim().min(1, "Give it a name"),
  url: z
    .string()
    .trim()
    // `URL` rather than a regex: it is the same parser the daemon uses, so a
    // url the form accepts is one the daemon can reach.
    .refine((value) => {
      try {
        const parsed = new URL(value);
        return parsed.protocol === "http:" || parsed.protocol === "https:";
      } catch {
        return false;
      }
    }, "Include the scheme, e.g. http://192.168.1.10:4533")
    // Empty is allowed: a fixed-url backend fills it in server-side, and the
    // form never showed the field.
    .or(z.literal("")),
  username: z.string().optional(),
  password: z.string().optional(),
});

export type AddServerValues = z.infer<typeof schema>;

export type AddServerFormProps = {
  isOpen: boolean;
  kinds: SourceKind[];
  /** Rejected by the daemon — a bad login, or a server that is not there. */
  error?: string;
  submitting?: boolean;
  onClose: () => void;
  onSubmit: (values: AddServerValues) => void;
};

/**
 * The desktop's add-server sheet: kind, name, url, and credentials for the
 * kinds that have any.
 *
 * The kind list comes from the daemon's provider registry rather than being
 * hardcoded here, so a backend added on the Rust side offers itself without
 * this form being touched.
 */
const AddServerForm = ({
  isOpen,
  kinds,
  error,
  submitting,
  onClose,
  onSubmit,
}: AddServerFormProps) => {
  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors },
  } = useForm<AddServerValues>({
    resolver: zodResolver(schema),
    defaultValues: { kind: kinds[0]?.kind ?? "", name: "", url: "" },
  });

  // Reopening must not show the last attempt's values.
  useEffect(() => {
    if (isOpen) reset({ kind: kinds[0]?.kind ?? "", name: "", url: "" });
  }, [isOpen, kinds, reset]);

  const kind = watch("kind");
  const selected = kinds.find((entry) => entry.kind === kind);
  const needsCredentials = selected?.needsCredentials ?? true;
  // A hosted backend has one address and it is not the user's to give, so the
  // field goes rather than being a thing to get wrong.
  const fixedUrl = selected?.fixedUrl ?? undefined;
  // Built from the registry's own default port, so a newly registered backend
  // gets a placeholder that points at the right one without this form being
  // told about it.
  const urlPlaceholder = `http://192.168.1.10:${selected?.defaultPort ?? 80}`;

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="Add server"
      icon={Icons.server}
      width={460}
      footer={
        <>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="submit"
            form="add-server"
            disabled={submitting}
            onClick={handleSubmit(onSubmit)}
          >
            {submitting ? "Connecting…" : "Add server"}
          </Button>
        </>
      }
    >
      <form
        id="add-server"
        className="flex flex-col gap-4 py-1"
        onSubmit={handleSubmit(onSubmit)}
      >
        <Select
          label="TYPE"
          options={kinds.map((entry) => ({
            value: entry.kind,
            label: entry.displayName,
          }))}
          error={errors.kind?.message}
          {...register("kind")}
        />
        <TextField
          label="NAME"
          placeholder="Living room NAS"
          error={errors.name?.message}
          {...register("name")}
        />
        {fixedUrl ? (
          <p className="text-[11px] text-muted">
            Connects to <span className="font-mono text-dim">{fixedUrl}</span>.
          </p>
        ) : (
          <TextField
            label="SERVER URL"
            placeholder={urlPlaceholder}
            error={errors.url?.message}
            {...register("url")}
          />
        )}
        {/* Hidden for backends with no login rather than asking for something
            that would be ignored. */}
        {needsCredentials && (
          <div className="flex gap-3">
            <TextField
              label="USERNAME"
              className="flex-1"
              error={errors.username?.message}
              {...register("username")}
            />
            <TextField
              label="PASSWORD"
              type="password"
              className="flex-1"
              error={errors.password?.message}
              {...register("password")}
            />
          </div>
        )}
        {error && (
          <p className="text-[11px] text-meter-high" role="alert">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
};

export default AddServerForm;
