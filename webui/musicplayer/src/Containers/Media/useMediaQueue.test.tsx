import { graphql, HttpResponse } from "msw";
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, makeQueryClient, renderWithProviders, screen } from "../../test/render";
import { server } from "../../test/server";
import type { MediaTrack } from "./api";
import { useMediaQueue } from "./useMediaQueue";

vi.mock("../../Hooks/usePlayback", () => ({ usePlayback: () => ({ playNext: vi.fn() }) }));
vi.mock("../../Hooks/usePlayTrack", () => ({ usePlayTrack: () => vi.fn() }));
afterEach(() => vi.restoreAllMocks());

const source = "mp-source:v1?resolver=emby&account=a&server=s&user=u&kind=item&id=episode";
const track: MediaTrack = { id: source, uri: source, title: "Episode", duration: 123, discNumber: 0, trackNumber: null };
function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => { resolve = done; });
  return { promise, resolve };
}
function Probe({ complete }: { complete: () => void }) {
  const actions = useMediaQueue("a");
  return <>
    <button disabled={actions.phase !== "idle"} onClick={() => {
      void actions.selected(track, { mediaSourceId: "version", audioStreamIndex: 0 }).then(complete);
    }}>Submit selected</button>
    <output>{actions.phase}</output>
    <p>{actions.message}</p>
    {actions.error && <p role="alert">{actions.error}</p>}
  </>;
}

describe("selected media submission", () => {
  it.each(["timeout", "source switch", "unmount"] as const)("bounds the request and ignores late success after %s", async (ending) => {
    const started = deferred();
    const release = deferred();
    const served = deferred();
    const complete = deferred();
    const submitted = vi.fn();
    server.use(graphql.mutation("AddSelectedMedia", async ({ variables }) => {
      submitted(variables);
      started.resolve();
      await release.promise;
      served.resolve();
      return HttpResponse.json({ data: { addSelectedMedia: ["occurrence-1"] } });
    }));
    const queryClient = makeQueryClient();
    queryClient.setQueryDefaults(["GetConnectedServer"], { gcTime: Infinity });
    queryClient.setQueryDefaults(["MediaBrowser"], { gcTime: Infinity });
    const account = (id: string) => ({ connectedServer: { id } });
    queryClient.setQueryData(["GetConnectedServer"], account("a"));
    queryClient.setQueryData(["MediaBrowser", "a"], { mediaBrowser: { serverId: "a", supported: true } });
    const originalTimeout = globalThis.setTimeout;
    let deadline: (() => void) | undefined;
    // Capture only the production deadline; MSW and React keep real timers.
    vi.spyOn(globalThis, "setTimeout").mockImplementation((callback, delay, ...args) => {
      if (delay === 15000) deadline = () => { if (typeof callback === "function") callback(...args); };
      return originalTimeout(callback, delay, ...args);
    });
    const view = renderWithProviders(<Probe complete={complete.resolve} />, { queryClient });
    try {
      fireEvent.click(screen.getByRole("button", { name: "Submit selected" }));
      await act(async () => { await started.promise; });
      expect(deadline).toBeDefined();
      expect(screen.getByRole("button")).toBeDisabled();
      await act(async () => {
        if (ending === "timeout") deadline!();
        else if (ending === "unmount") view.unmount();
        else {
          queryClient.setQueryData(["GetConnectedServer"], account("b"));
          queryClient.setQueryData(["GetConnectedServer"], account("a"));
        }
        await complete.promise;
      });
      if (ending === "timeout") {
        expect(screen.getByRole("alert")).toHaveTextContent("outcome uncertain");
        expect(screen.getByRole("alert")).toHaveTextContent("Do not retry this mutation");
      } else expect(screen.queryByRole("alert")).not.toBeInTheDocument();
      if (ending !== "unmount") expect(screen.getByRole("button")).toBeEnabled();
      await act(async () => { release.resolve(); await served.promise; });
      expect(screen.queryByText("Added to the queue with the selected audio.")).not.toBeInTheDocument();
      expect(submitted).toHaveBeenCalledExactlyOnceWith({ entries: [{
        track, choice: { mediaSourceId: "version", audioStreamIndex: 0 },
      }] });
    } finally {
      release.resolve();
      view.unmount();
      queryClient.clear();
    }
  });
});
