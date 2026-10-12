import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import { act, renderWithProviders, screen, waitFor, within } from "../../test/render";
import { server } from "../../test/server";
import type { MediaQueueOccurrence } from "./api";
import MediaQueueAudio from "./MediaQueueAudio";

const source = "mp-source:v1?resolver=emby&account=a&server=s&user=u&kind=item&id=55508";
const entry = (occurrenceId: string): MediaQueueOccurrence => ({
  occurrenceId,
  track: { id: source, uri: source, title: "Same episode", duration: 19142, discNumber: 0, trackNumber: null },
  choice: { mediaSourceId: "version", audioStreamIndex: 0 }, acceptedAudio: null,
});

describe("queue audio editing", () => {
  it("retains an uncertain mutation result when the following queue refresh fails or loads", async () => {
    const uncertain = "Player acknowledgement timed out; outcome uncertain. The delayed command may still apply. Do not retry this mutation.";
    let mutated = false;
    let holdRefresh = false;
    let release!: () => void;
    const held = new Promise<void>((resolve) => { release = resolve; });
    const update = vi.fn();
    server.use(
      graphql.query("MediaQueue", async () => {
        if (holdRefresh) await held;
        if (mutated) return HttpResponse.json({ errors: [{ message: "Queue refresh unavailable" }] });
        return HttpResponse.json({ data: { mediaQueue: { current: entry("occurrence-1"), played: [], upcoming: [] } } });
      }),
      graphql.query("MediaAudioOptions", () => HttpResponse.json({ data: { mediaAudioOptions: {
        source, versions: [{ id: "version", name: "Original", runtimeTicks: null,
          defaultAudioStreamIndex: 0, unavailableReason: null, audioStreams: [{
            index: 0, title: "English", displayTitle: null, language: "en", codec: "aac",
            channels: 2, sampleRate: 44100, isDefault: true, unavailableReason: null,
          }] }],
      } } })),
      graphql.mutation("SelectMediaAudio", () => {
        update();
        mutated = true;
        return HttpResponse.json({ errors: [{ message: uncertain }] });
      }),
    );
    const view = renderWithProviders(<MediaQueueAudio />);
    try {
      await view.user.click(await screen.findByRole("button", { name: "Change audio" }));
      await view.user.click(await screen.findByRole("button", { name: "Save audio choice" }));
      expect(await screen.findByText("Queue refresh unavailable")).toBeInTheDocument();
      expect(screen.getByText(uncertain)).toHaveAttribute("role", "alert");
      // Reset only the read cache to exercise the loading branch without
      // remounting (which would discard the mutation's outcome).
      holdRefresh = true;
      await act(async () => {
        void view.queryClient.resetQueries({ queryKey: ["MediaQueue"], exact: true });
      });
      expect(await screen.findByText("Loading queue audio…")).toBeInTheDocument();
      expect(screen.getByText(uncertain)).toHaveAttribute("role", "alert");
      release();
      expect(await screen.findByText("Queue refresh unavailable")).toBeInTheDocument();
      expect(screen.getByText(uncertain)).toBeInTheDocument();
      expect(update).toHaveBeenCalledTimes(1);
    } finally {
      release();
      view.unmount();
      view.queryClient.clear();
    }
  });

  it("edits the selected occurrence of a repeated episode, preserving index zero", async () => {
    const first = entry("11111111-1111-4111-8111-111111111111");
    const second = entry("22222222-2222-4222-8222-222222222222");
    const update = vi.fn();
    server.use(
      graphql.query("MediaQueue", () => HttpResponse.json({ data: { mediaQueue: {
        current: first, played: [first], upcoming: [second],
      } } })),
      graphql.query("MediaAudioOptions", () => HttpResponse.json({ data: { mediaAudioOptions: {
        source, versions: [{ id: "version", name: "Original", runtimeTicks: "191429666670",
          defaultAudioStreamIndex: 0, unavailableReason: null, audioStreams: [{
            index: 0, title: "English", displayTitle: null, language: "en", codec: "aac",
            channels: 2, sampleRate: 44100, isDefault: false, unavailableReason: null,
          }] }],
      } } })),
      graphql.mutation("SelectMediaAudio", ({ variables }) => {
        update(variables);
        return HttpResponse.json({ data: { selectMediaAudio: true } });
      }),
    );
    const { user } = renderWithProviders(<MediaQueueAudio />);
    const secondRow = await screen.findByRole("region", { name: "Up next 1: Same episode" });
    expect(screen.getAllByText("Same episode")).toHaveLength(2);
    await user.click(within(secondRow).getByRole("button", { name: "Change audio" }));
    expect(await within(secondRow).findByRole("combobox")).toHaveValue(JSON.stringify(["version", 0]));
    await user.click(within(secondRow).getByRole("button", { name: "Save audio choice" }));
    await waitFor(() => expect(update).toHaveBeenCalledExactlyOnceWith({
      occurrenceId: second.occurrenceId, choice: { mediaSourceId: "version", audioStreamIndex: 0 },
    }));
    await waitFor(() => expect(screen.queryByRole("combobox")).not.toBeInTheDocument());
    const currentRow = screen.getByRole("region", { name: "Current: Same episode" });
    await user.click(within(currentRow).getByRole("button", { name: "Change audio" }));
    expect(within(currentRow).getByText(/restarts the current item from the beginning/)).toBeInTheDocument();
  });

  it("surfaces an unsupported queue API without falling back to track-ID editing", async () => {
    server.use(graphql.query("MediaQueue", () => HttpResponse.json({
      errors: [{ message: "Audio choices are unavailable on this remote receiver" }],
    })));
    renderWithProviders(<MediaQueueAudio />);
    expect(await screen.findByRole("alert")).toHaveTextContent("unavailable on this remote receiver");
    expect(screen.queryByRole("button", { name: "Change audio" })).not.toBeInTheDocument();
  });
});
