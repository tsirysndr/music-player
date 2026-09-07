import { zodResolver } from "@hookform/resolvers/zod";
import { FC, useCallback, useEffect, useRef, useState } from "react";
import { Controller, useForm } from "react-hook-form";
import { z } from "zod";
import { fetcher } from "../../../Api/fetcher";
import RsqlEditor from "../../RsqlEditor";
import { FIELDS } from "../../RsqlEditor/fields";
import {
  Button,
  Dialog,
  Icons,
  Select,
  TextAreaField,
  TextField,
  Toggle,
  cn,
} from "../../UI";

/**
 * The form's shape. A smart playlist needs nothing beyond a name — an empty
 * filter legitimately means "the whole library" — so the schema's only real
 * job is to insist on a name and to keep the limit a sane number.
 */
const schema = z.object({
  name: z.string().trim().min(1, "Give your playlist a name"),
  description: z.string().optional(),
  smart: z.boolean(),
  filter: z.string(),
  sortBy: z.string(),
  sortOrder: z.enum(["asc", "desc"]),
  limit: z
    .string()
    .refine((value) => value === "" || /^\d+$/.test(value), "Must be a number")
    .refine(
      (value) => value === "" || Number(value) <= 10000,
      "That is more tracks than a playlist should hold"
    ),
});

type FormValues = z.infer<typeof schema>;

export type SmartPlaylistRule = {
  filter: string;
  sortBy?: string;
  sortOrder?: string;
  limit?: number;
};

export type NewPlaylistModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onCreatePlaylist: (
    name: string,
    description?: string,
    smart?: SmartPlaylistRule
  ) => void;
};

/**
 * The readable part of whatever the fetcher threw. A GraphQL error arrives as
 * `{ response: { errors: [{ message }] } }` rather than an `Error`.
 */
function errorMessage(thrown: unknown): string {
  if (thrown instanceof Error) return thrown.message;
  const errors = (thrown as any)?.response?.errors;
  if (Array.isArray(errors) && errors[0]?.message) {
    return String(errors[0].message);
  }
  return "Could not check the filter";
}

const SORT_OPTIONS = [
  { value: "", label: "Library order" },
  { value: "random", label: "Random" },
  ...FIELDS.map((field) => ({ value: field.name, label: field.label })),
];

const ORDER_OPTIONS = [
  { value: "asc", label: "Ascending" },
  { value: "desc", label: "Descending" },
];

const NewPlaylistModal: FC<NewPlaylistModalProps> = ({
  onClose,
  isOpen,
  onCreatePlaylist,
}) => {
  const {
    control,
    register,
    handleSubmit,
    reset,
    watch,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      name: "",
      description: "",
      smart: false,
      filter: "",
      sortBy: "",
      sortOrder: "asc",
      limit: "",
    },
  });

  const smart = watch("smart");
  const filter = watch("filter");
  const sortBy = watch("sortBy");
  const sortOrder = watch("sortOrder");
  const limit = watch("limit");

  const [preview, setPreview] = useState<{
    count: number | null;
    error: string;
    loading: boolean;
  }>({ count: null, error: "", loading: false });

  // Only the newest preview may write the status: a filter typed quickly
  // leaves several in flight, and a slow early one must not overwrite a later
  // verdict.
  const previewId = useRef(0);

  const runPreview = useCallback(async () => {
    const id = ++previewId.current;
    setPreview((p) => ({ ...p, loading: true }));
    try {
      const data = await fetcher<any, any>(
        `query($smart:SmartPlaylistInput!){smartPlaylistPreview(smart:$smart,sample:0){count}}`,
        {
          smart: {
            filter,
            sortBy: sortBy || null,
            sortOrder,
            limit: limit ? Number(limit) : null,
          },
        }
      )();
      if (id !== previewId.current) return;
      setPreview({
        count: data.smartPlaylistPreview.count,
        error: "",
        loading: false,
      });
    } catch (e) {
      if (id !== previewId.current) return;
      setPreview({
        count: null,
        // The fetcher surfaces a GraphQL error as a plain object, so the
        // message has to be dug out rather than assumed.
        error: errorMessage(e),
        loading: false,
      });
    }
    // The order changes the sequence, not how many match, so it needs no
    // extra round trip — but it is in the deps so the closure stays current.
  }, [filter, sortBy, sortOrder, limit]);

  // Check the filter a beat after typing stops. The count is a round trip, and
  // a filter is invalid for most of the time it is being written.
  useEffect(() => {
    if (!smart) return;
    const timer = window.setTimeout(runPreview, 500);
    return () => window.clearTimeout(timer);
  }, [smart, filter, sortBy, limit, runPreview]);

  const submit = (values: FormValues) => {
    onCreatePlaylist(
      values.name.trim(),
      values.description,
      values.smart
        ? {
            filter: values.filter,
            sortBy: values.sortBy || undefined,
            sortOrder: values.sortOrder,
            limit: values.limit ? Number(values.limit) : undefined,
          }
        : undefined
    );
    close();
  };

  const close = () => {
    onClose();
    reset();
    setPreview({ count: null, error: "", loading: false });
  };

  const status = preview.loading
    ? { tone: "muted" as const, text: "Checking…" }
    : preview.error
      ? { tone: "error" as const, text: preview.error }
      : preview.count === null
        ? {
            tone: "muted" as const,
            text: `Fields: ${FIELDS.map((f) => f.name).join(", ")}`,
          }
        : {
            tone: "ok" as const,
            text:
              preview.count === 1
                ? "1 track matches"
                : `${preview.count} tracks match`,
          };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title="Create new playlist"
      icon={Icons.playlist}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button onClick={handleSubmit(submit)}>Create playlist</Button>
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

        <Controller
          control={control}
          name="smart"
          render={({ field: { value, onChange } }) => (
            <div className="flex items-start gap-3">
              <Toggle
                checked={value}
                label="Smart playlist"
                onChange={onChange}
              />
              <div>
                <p className="text-[13px] text-fg">Smart playlist</p>
                <p className="mt-[2px] text-[11px] text-muted">
                  Fills itself from a filter, and keeps itself up to date as
                  your library changes.
                </p>
              </div>
            </div>
          )}
        />

        {smart && (
          <>
            <div>
              <p className="mb-[5px] text-[10px] tracking-[1px] text-muted">
                FILTER
              </p>
              <Controller
                control={control}
                name="filter"
                render={({ field: { value, onChange } }) => (
                  <RsqlEditor
                    value={value}
                    onChange={onChange}
                    onCommit={runPreview}
                    placeholder="genre==rock;year>2000"
                  />
                )}
              />
              <p
                className={cn(
                  "mt-[6px] min-h-4 text-[11px]",
                  status.tone === "error"
                    ? "text-syntax-error"
                    : status.tone === "ok"
                      ? "text-meter-low"
                      : "text-muted"
                )}
              >
                {status.text}
              </p>
            </div>

            <div className="flex flex-col gap-3 sm:flex-row">
              <Select
                label="SORT BY"
                options={SORT_OPTIONS}
                className="flex-1"
                {...register("sortBy")}
              />
              <Select
                label="ORDER"
                options={ORDER_OPTIONS}
                className="flex-1"
                {...register("sortOrder")}
              />
              <TextField
                label="LIMIT"
                placeholder="0 = all"
                inputMode="numeric"
                className="flex-1"
                error={errors.limit?.message}
                {...register("limit")}
              />
            </div>
          </>
        )}
      </form>
    </Dialog>
  );
};

export default NewPlaylistModal;
