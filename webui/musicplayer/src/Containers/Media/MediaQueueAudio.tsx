import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Button } from "../../Components/UI";
import AudioChoiceEditor from "./AudioChoiceEditor";
import { fetchMediaQueue, selectMediaAudio, type AudioChoice, type MediaQueueOccurrence } from "./api";
import { errorMessage } from "./useMediaQueue";

export default function MediaQueueAudio() {
  const client = useQueryClient();
  const [editing, setEditing] = useState<string | null>(null);
  const queue = useQuery({
    queryKey: ["MediaQueue"], queryFn: ({ signal }) => fetchMediaQueue(signal),
    retry: false, refetchInterval: (query) => query.state.status === "error" ? false : 2000,
  });
  const update = useMutation({
    mutationFn: async ({ occurrenceId, choice }: { occurrenceId: string; choice: AudioChoice }) => {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), 15000);
      try {
        await selectMediaAudio(occurrenceId, choice, controller.signal);
      } catch (error) {
        if (controller.signal.aborted) throw new Error("The audio update timed out and may still apply. Do not resubmit it based on an unchanged queue.");
        throw error;
      } finally { clearTimeout(timer); }
    },
    retry: false,
    onSuccess: () => { setEditing(null); },
    onSettled: () => {
      void client.invalidateQueries({ queryKey: ["MediaQueue"] });
      void client.invalidateQueries({ queryKey: ["GetTracklist"] });
    },
  });
  const mutationError = update.isError && <p role="alert">{errorMessage(update.error)}</p>;
  if (queue.isPending) return <div className="space-y-2 p-3">
    {mutationError}
    <p role="status" className="text-sm">Loading queue audio…</p>
  </div>;
  if (queue.isError) return <div className="space-y-2 p-3">
    {mutationError}
    <p role="alert">{errorMessage(queue.error)}</p>
    <Button onClick={() => { void queue.refetch(); }}>Refresh queue audio</Button>
  </div>;
  const current = queue.data.current;
  const entries = [
    ...(current ? [{ entry: current, label: "Current" }] : []),
    ...queue.data.upcoming.map((entry, index) => ({ entry, label: `Up next ${index + 1}` })),
    ...queue.data.played.filter((entry) => entry.occurrenceId !== current?.occurrenceId)
      .map((entry, index) => ({ entry, label: `History ${index + 1}` })),
  ].filter(({ entry }) => entry.track.uri?.startsWith("mp-source:v1?"));
  const selectedLabel = (entry: MediaQueueOccurrence) => {
    const audio = entry.acceptedAudio ?? (entry.choice.mediaSourceId !== null ? entry.choice : null);
    return audio ? `${audio.mediaSourceId} · Audio ${audio.audioStreamIndex}` : "Automatic";
  };
  return <div className="space-y-3 p-3">
    {mutationError}
    {entries.length === 0 && <p className="text-sm text-muted">No media audio choices in this queue.</p>}
    {entries.map(({ entry, label }) => <section key={entry.occurrenceId} aria-label={`${label}: ${entry.track.title}`} className="space-y-2 rounded-control border border-line p-2">
      <p className="text-xs text-muted">{label}</p>
      <p className="text-sm">{entry.track.title}</p>
      <p className="break-all text-xs text-muted">{selectedLabel(entry)}</p>
      <Button disabled={update.isPending} variant="outline" onClick={() => setEditing(editing === entry.occurrenceId ? null : entry.occurrenceId)}>
        {editing === entry.occurrenceId ? "Close audio choices" : "Change audio"}
      </Button>
      {editing === entry.occurrenceId && <>
        {entry.occurrenceId === current?.occurrenceId && <p className="text-xs text-muted">Changing this audio restarts the current item from the beginning.</p>}
        <AudioChoiceEditor key={entry.occurrenceId} source={entry.track.uri!} busy={update.isPending}
          initialChoice={entry.choice.mediaSourceId !== null && entry.choice.audioStreamIndex !== null
            ? { mediaSourceId: entry.choice.mediaSourceId, audioStreamIndex: entry.choice.audioStreamIndex } : {}}
          submitLabel="Save audio choice"
          onSubmit={(choice) => update.mutate({ occurrenceId: entry.occurrenceId, choice })} />
      </>}
    </section>)}
  </div>;
}
