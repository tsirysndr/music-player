import styled from "@emotion/styled";
import { useTheme } from "@emotion/react";
import { zodResolver } from "@hookform/resolvers/zod";
import { Input } from "baseui/input";
import { Modal, ModalHeader, ModalBody, ModalFooter } from "baseui/modal";
import { Textarea } from "baseui/textarea";
import { Checkbox } from "baseui/checkbox";
import { Select } from "baseui/select";
import { FC, useCallback, useEffect, useRef, useState } from "react";
import { Controller, useForm } from "react-hook-form";
import { z } from "zod";
import Button from "../../Button";
import RsqlEditor from "../../RsqlEditor";
import { FIELDS } from "../../RsqlEditor/fields";
import { fetcher } from "../../../Api/fetcher";

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
      "That is more tracks than a playlist should hold",
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
    smart?: SmartPlaylistRule,
  ) => void;
};

const Field = styled.div`
  margin-bottom: 16px;
`;

const Label = styled.div`
  font-size: 12px;
  color: ${(props) => props.theme.colors.secondaryText};
  margin-bottom: 6px;
`;

const FieldError = styled.div`
  color: #d45769;
  font-size: 12px;
  margin-top: 5px;
`;

const Status = styled.div<{ tone: "ok" | "error" | "muted" }>`
  font-size: 12px;
  margin-top: 6px;
  min-height: 16px;
  color: ${(props) =>
    props.tone === "error"
      ? "#d45769"
      : props.tone === "ok"
      ? "#2f9e6e"
      : props.theme.colors.secondaryText};
`;

const Row = styled.div`
  display: flex;
  gap: 12px;

  & > * {
    flex: 1;
  }
`;

const SmartRow = styled.div`
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin: 4px 0 18px;
`;

const SmartHint = styled.div`
  font-size: 12px;
  color: ${(props) => props.theme.colors.secondaryText};
  margin-top: 2px;
`;

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
  { id: "", label: "Library order" },
  { id: "random", label: "Random" },
  ...FIELDS.map((field) => ({ id: field.name, label: field.label })),
];

const inputOverrides = (theme: any) => ({
  Root: {
    style: ({ $isFocused }: { $isFocused: boolean }) => ({
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
      backgroundColor: theme.colors.popoverBackground,
      fontSize: "14px",
      paddingLeft: "0px !important",
      paddingRight: "0px !important",
    },
  },
  InputContainer: {
    style: { backgroundColor: theme.colors.popoverBackground },
  },
});

const NewPlaylistModal: FC<NewPlaylistModalProps> = ({
  onClose,
  isOpen,
  onCreatePlaylist,
}) => {
  const theme = useTheme();
  const {
    control,
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
        },
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
        : undefined,
    );
    onClose();
    reset();
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
    <Modal onClose={close} isOpen={isOpen}>
      <ModalHeader>Create new playlist</ModalHeader>
      <ModalBody>
        <Field>
          <Controller
            control={control}
            name="name"
            render={({ field }) => (
              <Input
                {...field}
                placeholder="Give your playlist a title"
                overrides={inputOverrides(theme)}
              />
            )}
          />
          {errors.name && <FieldError>{errors.name.message}</FieldError>}
        </Field>

        <Field>
          <Controller
            control={control}
            name="description"
            render={({ field }) => (
              <Textarea
                {...field}
                placeholder="Write a description"
                overrides={inputOverrides(theme)}
              />
            )}
          />
        </Field>

        <SmartRow>
          <Controller
            control={control}
            name="smart"
            render={({ field: { value, onChange, ...rest } }) => (
              <Checkbox
                {...rest}
                checked={value}
                onChange={(e) => onChange(e.currentTarget.checked)}
              >
                Smart playlist
              </Checkbox>
            )}
          />
        </SmartRow>
        {!smart && (
          <SmartHint style={{ marginTop: -14, marginBottom: 14 }}>
            A smart playlist fills itself from a filter, and keeps itself up to
            date as your library changes.
          </SmartHint>
        )}

        {smart && (
          <>
            <Field>
              <Label>Filter</Label>
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
              <Status tone={status.tone}>{status.text}</Status>
            </Field>

            <Row>
              <Field>
                <Label>Sort by</Label>
                <Controller
                  control={control}
                  name="sortBy"
                  render={({ field: { value, onChange } }) => (
                    <Select
                      options={SORT_OPTIONS}
                      value={SORT_OPTIONS.filter((o) => o.id === value)}
                      onChange={({ value: selected }) =>
                        onChange(selected[0]?.id ?? "")
                      }
                      clearable={false}
                      searchable={false}
                    />
                  )}
                />
              </Field>
              <Field>
                <Label>Order</Label>
                <Controller
                  control={control}
                  name="sortOrder"
                  render={({ field: { value, onChange } }) => (
                    <Select
                      options={[
                        { id: "asc", label: "Ascending" },
                        { id: "desc", label: "Descending" },
                      ]}
                      value={[{ id: value, label: value === "desc" ? "Descending" : "Ascending" }]}
                      onChange={({ value: selected }) =>
                        onChange(selected[0]?.id ?? "asc")
                      }
                      clearable={false}
                      searchable={false}
                    />
                  )}
                />
              </Field>
              <Field>
                <Label>Limit</Label>
                <Controller
                  control={control}
                  name="limit"
                  render={({ field }) => (
                    <Input
                      {...field}
                      placeholder="0 = all"
                      overrides={inputOverrides(theme)}
                    />
                  )}
                />
                {errors.limit && <FieldError>{errors.limit.message}</FieldError>}
              </Field>
            </Row>
          </>
        )}
      </ModalBody>
      <ModalFooter>
        <Button onClick={handleSubmit(submit)}>Create New</Button>
      </ModalFooter>
    </Modal>
  );
};

export default NewPlaylistModal;
