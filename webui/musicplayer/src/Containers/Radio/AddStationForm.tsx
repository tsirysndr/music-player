import { zodResolver } from "@hookform/resolvers/zod";
import { FC, useRef, useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { fetcher } from "../../Api/fetcher";
import { Button, Dialog, Icons, TextField, cn } from "../../Components/UI";
import { STATION_FIELDS, type Station } from "./api";

const schema = z.object({
  streamUrl: z
    .string()
    .trim()
    .min(1, "A station needs a stream url")
    .url("That does not look like a url"),
  name: z.string().trim().min(1, "Give the station a name"),
  genre: z.string().optional(),
  country: z.string().optional(),
  logo: z
    .string()
    .trim()
    .refine(
      (value) => value === "" || /^https?:\/\//.test(value),
      "A logo has to be an http(s) url"
    )
    .optional(),
});

type FormValues = z.infer<typeof schema>;

export type AddStationFormProps = {
  isOpen: boolean;
  onClose: () => void;
  onAdded: (station: Station) => void;
};

/**
 * The "add station" form. The stream url is checked against the station itself
 * before anything is saved — an unreachable url is the one mistake that makes
 * a station useless — and a station that announces itself over ICY fills in
 * its own name, genre and bitrate.
 */
const AddStationForm: FC<AddStationFormProps> = ({
  isOpen,
  onClose,
  onAdded,
}) => {
  const {
    register,
    handleSubmit,
    reset,
    setValue,
    getValues,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      streamUrl: "",
      name: "",
      genre: "",
      country: "",
      logo: "",
    },
  });

  const [checking, setChecking] = useState(false);
  const [status, setStatus] = useState<{
    tone: "ok" | "error" | "muted";
    text: string;
  }>({ tone: "muted", text: "" });

  // Only the newest check may write the status: a url typed quickly leaves
  // several in flight, and a slow early one must not overwrite a later verdict.
  const checkId = useRef(0);

  const checkUrl = async (url: string) => {
    const id = ++checkId.current;
    if (!url.trim()) {
      setStatus({ tone: "muted", text: "" });
      return;
    }
    setChecking(true);
    try {
      const data = await fetcher<any, any>(
        `query($url:String!){checkRadioStream(url:$url){ok error name genre bitrate codec}}`,
        { url }
      )();
      const check = data.checkRadioStream;
      if (id !== checkId.current) return;
      if (!check.ok) {
        setStatus({ tone: "error", text: check.error });
        return;
      }
      setStatus({
        tone: "ok",
        text: [
          "Stream reachable",
          check.codec,
          check.bitrate ? `${check.bitrate} kbps` : "",
        ]
          .filter(Boolean)
          .join(" · "),
      });
      // Only fill in what the user has not typed themselves.
      if (!getValues("name") && check.name) setValue("name", check.name);
      if (!getValues("genre") && check.genre) setValue("genre", check.genre);
    } catch (e) {
      if (id !== checkId.current) return;
      setStatus({
        tone: "error",
        text: e instanceof Error ? e.message : "Could not check the stream",
      });
    } finally {
      if (id === checkId.current) setChecking(false);
    }
  };

  const close = () => {
    onClose();
    reset();
    setStatus({ tone: "muted", text: "" });
  };

  const submit = async (values: FormValues) => {
    try {
      const data = await fetcher<any, any>(
        `mutation($station:NewRadioStationInput!){addRadioStation(station:$station){${STATION_FIELDS}}}`,
        { station: values }
      )();
      onAdded(data.addRadioStation);
      close();
    } catch (e) {
      setStatus({
        tone: "error",
        text: e instanceof Error ? e.message : "Could not add the station",
      });
    }
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title="Add a station"
      icon={Icons.broadcast}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button disabled={isSubmitting} onClick={handleSubmit(submit)}>
            {isSubmitting ? "Saving…" : "Add station"}
          </Button>
        </>
      }
    >
      <p className="mb-4 text-[11px] text-muted">
        Signed in to atradio.fm, it is published to your account and follows you
        to your other devices.
      </p>
      <form
        onSubmit={handleSubmit(submit)}
        className="flex flex-col gap-4 pb-1"
      >
        <div>
          <TextField
            label="STREAM URL"
            autoFocus
            placeholder="https://example.com/stream"
            error={errors.streamUrl?.message}
            {...register("streamUrl", {
              onBlur: (event) => checkUrl(event.target.value),
              onChange: () => setStatus({ tone: "muted", text: "" }),
            })}
          />
          <p
            className={cn(
              "mt-[6px] min-h-4 text-[11px]",
              checking || status.tone === "muted"
                ? "text-muted"
                : status.tone === "error"
                  ? "text-syntax-error"
                  : "text-meter-low"
            )}
          >
            {checking ? "Checking the stream…" : status.text}
          </p>
        </div>
        <TextField
          label="NAME"
          placeholder="Station name"
          error={errors.name?.message}
          {...register("name")}
        />
        <div className="flex flex-col gap-4 sm:flex-row">
          <TextField
            label="GENRE"
            placeholder="jazz"
            className="flex-1"
            {...register("genre")}
          />
          <TextField
            label="COUNTRY"
            placeholder="France"
            className="flex-1"
            {...register("country")}
          />
        </div>
        <TextField
          label="LOGO URL"
          placeholder="https://example.com/logo.png"
          error={errors.logo?.message}
          {...register("logo")}
        />
      </form>
    </Dialog>
  );
};

export default AddStationForm;
