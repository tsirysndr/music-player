import { useQuery } from "@tanstack/react-query";
import { useId, useState } from "react";
import { Button } from "../../Components/UI";
import { fetchAudioOptions, type AudioChoice } from "./api";
import { errorMessage } from "./useMediaQueue";

export default function AudioChoiceEditor({ source, busy, onSubmit, initialChoice = {}, submitLabel = "Add with this audio" }: {
  source: string; busy: boolean; onSubmit: (choice: AudioChoice) => void; submitLabel?: string; initialChoice?: AudioChoice;
}) {
  const labelId = useId();
  const [selected, setSelected] = useState(() => initialChoice.mediaSourceId != null && initialChoice.audioStreamIndex != null
    ? JSON.stringify([initialChoice.mediaSourceId, initialChoice.audioStreamIndex]) : "auto");
  const options = useQuery({
    queryKey: ["MediaAudioOptions", source],
    queryFn: ({ signal }) => fetchAudioOptions(source, signal),
    retry: false,
  });
  if (options.isPending) return <p role="status" className="text-sm text-muted">Loading audio tracks…</p>;
  if (options.isError) return <div className="space-y-2">
    <p role="alert">{errorMessage(options.error)}</p>
    <Button variant="outline" onClick={() => { void options.refetch(); }}>Retry audio tracks</Button>
  </div>;
  const choices = options.data.versions.flatMap((version) =>
    version.audioStreams.map((stream) => {
      const reason = version.unavailableReason ?? stream.unavailableReason;
      const isDefault = version.defaultAudioStreamIndex !== null
        ? version.defaultAudioStreamIndex === stream.index : stream.isDefault;
      const label = [version.name || version.id,
        stream.displayTitle || stream.title || stream.language || `Audio ${stream.index}`,
        stream.codec, stream.channels ? `${stream.channels} channels` : null,
        isDefault ? "Default" : null, reason,
      ].filter(Boolean).join(" · ");
      return { key: JSON.stringify([version.id, stream.index]), label, disabled: !!reason,
        choice: { mediaSourceId: version.id, audioStreamIndex: stream.index } };
    }));
  const selectable = choices.some((option) => !option.disabled);
  const explicit = choices.find((option) => option.key === selected);
  const valid = selected === "auto" ? selectable : !!explicit && !explicit.disabled;
  return <div className="space-y-2 rounded-control border border-line p-3">
    <label id={labelId} className="block text-sm">Media version and audio track</label>
    <select aria-labelledby={labelId} value={selected} disabled={busy || !selectable}
      className="w-full rounded-control border border-line bg-panel p-2 text-sm"
      onChange={(event) => setSelected(event.target.value)}>
      <option value="auto">Automatic</option>
      {selected !== "auto" && !explicit && <option value={selected} disabled>Selected audio is no longer available</option>}
      {choices.map((option, index) => <option key={`${option.key}:${index}`} value={option.key} disabled={option.disabled}>
        {option.label}
      </option>)}
    </select>
    {!selectable && <p role="status" className="text-sm text-muted">No supported audio tracks are available.</p>}
    {selected !== "auto" && !valid && <p role="alert">The selected audio is unavailable. Choose another audio track.</p>}
    <Button disabled={busy || !valid} onClick={() => onSubmit(selected === "auto" ? {} : explicit!.choice)}>
      {busy ? "Updating queue…" : submitLabel}
    </Button>
  </div>;
}
