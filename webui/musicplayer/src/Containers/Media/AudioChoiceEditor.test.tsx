import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import { act, renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import type { AudioOptions } from "./api";
import AudioChoiceEditor from "./AudioChoiceEditor";

const source = "mp-source:v1?resolver=emby&account=a&server=s&user=u&kind=item&id=55508";
const stream = (index: number, title: string): AudioOptions["versions"][number]["audioStreams"][number] => ({
  index, title, displayTitle: null, language: "en", codec: "aac",
  channels: 2, sampleRate: 44100, isDefault: false, unavailableReason: null,
});
const fixture = (): AudioOptions => ({
  source, versions: [{
    id: "actual-version", name: "Original", runtimeTicks: "191429666670",
    defaultAudioStreamIndex: 0, unavailableReason: null,
    audioStreams: [stream(7, "Commentary"), stream(0, "English")],
  }, {
    id: "live-version", name: "Live", runtimeTicks: null,
    defaultAudioStreamIndex: null, unavailableReason: "Live playback is unsupported",
    audioStreams: [stream(0, "Live audio")],
  }],
});

describe("audio choice", () => {
  it("submits the real version and index zero and honours the declared default", async () => {
    server.use(graphql.query("MediaAudioOptions", () => HttpResponse.json({
      data: { mediaAudioOptions: fixture() },
    })));
    const submit = vi.fn();
    const { user } = renderWithProviders(<AudioChoiceEditor source={source} busy={false} onSubmit={submit} />);
    const select = await screen.findByRole("combobox", { name: "Media version and audio track" });
    expect(screen.getByRole("option", { name: /English.*Default/ })).toBeEnabled();
    expect(screen.getByRole("option", { name: /Live audio/ })).toBeDisabled();
    await user.selectOptions(select, JSON.stringify(["actual-version", 0]));
    await user.click(screen.getByRole("button", { name: "Add with this audio" }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ mediaSourceId: "actual-version", audioStreamIndex: 0 });
  });

  it("keeps the selected identity across reordering and requires a new choice if it disappears", async () => {
    let options = fixture();
    server.use(graphql.query("MediaAudioOptions", () => HttpResponse.json({
      data: { mediaAudioOptions: options },
    })));
    const submit = vi.fn();
    const { user, queryClient } = renderWithProviders(<AudioChoiceEditor source={source} busy={false} onSubmit={submit} />);
    const select = await screen.findByRole("combobox");
    const selected = JSON.stringify(["actual-version", 0]);
    await user.selectOptions(select, selected);
    options = { ...options, versions: options.versions.map((version) => ({
      ...version, audioStreams: [...version.audioStreams].reverse(),
    })).reverse() };
    await act(async () => {
      await queryClient.invalidateQueries({ queryKey: ["MediaAudioOptions", source] });
    });
    // Refetch completion can precede the query observer's React update.
    await waitFor(() => expect(screen.getAllByRole("option").map((option) => option.getAttribute("value"))).toEqual([
      "auto", JSON.stringify(["live-version", 0]), selected, JSON.stringify(["actual-version", 7]),
    ]));
    expect(select).toHaveValue(selected);
    await user.click(screen.getByRole("button", { name: "Add with this audio" }));
    expect(submit).toHaveBeenCalledExactlyOnceWith({ mediaSourceId: "actual-version", audioStreamIndex: 0 });
    options = { ...options, versions: options.versions.map((version) => ({
      ...version, audioStreams: version.audioStreams.filter((audio) => audio.index !== 0),
    })) };
    await act(async () => {
      await queryClient.invalidateQueries({ queryKey: ["MediaAudioOptions", source] });
    });
    expect(await screen.findByRole("alert")).toHaveTextContent("The selected audio is unavailable");
    expect(select).toHaveValue(selected);
    expect(screen.getByRole("button", { name: "Add with this audio" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Add with this audio" }));
    expect(submit).toHaveBeenCalledTimes(1);
    await user.selectOptions(select, "auto");
    expect(screen.getByRole("button", { name: "Add with this audio" })).toBeEnabled();
  });

  it("shows unsupported API errors without submitting an automatic fallback", async () => {
    server.use(graphql.query("MediaAudioOptions", () => HttpResponse.json({
      errors: [{ message: "Audio selection is unsupported" }],
    })));
    const submit = vi.fn();
    renderWithProviders(<AudioChoiceEditor source={source} busy={false} onSubmit={submit} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Audio selection is unsupported");
    expect(screen.queryByRole("button", { name: "Add with this audio" })).not.toBeInTheDocument();
    expect(submit).not.toHaveBeenCalled();
  });
});
