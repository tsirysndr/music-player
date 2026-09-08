import { graphql, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";
import * as fixtures from "../../test/handlers";
import { renderWithProviders, screen, waitFor, within } from "../../test/render";
import { server } from "../../test/server";
import AppShell from "./AppShell";

/** One row from the connected server, one from this machine. */
const federated = () =>
  graphql.query("Search", () =>
    HttpResponse.json({
      data: {
        search: {
          __typename: "SearchResult",
          artists: [],
          albums: [],
          tracks: [
            {
              __typename: "Track",
              id: "remote-1",
              title: "God Is Dead?",
              artist: "Black Sabbath",
              duration: 532,
              cover: null,
              artistId: "a1",
              albumId: "al1",
              albumTitle: "13",
              source: "Living room NAS",
            },
            {
              __typename: "Track",
              id: "local-1",
              title: "God Is Dead? (local)",
              artist: "Black Sabbath",
              duration: 532,
              cover: null,
              artistId: "a2",
              albumId: "al2",
              albumTitle: "13",
              source: null,
            },
          ],
        },
      },
    })
  );

const connected = () =>
  graphql.query("GetSavedServers", () =>
    HttpResponse.json({
      data: {
        savedServers: fixtures.savedServers.map((entry, index) => ({
          ...entry,
          connected: index === 0,
        })),
      },
    })
  );

const openPalette = async () => {
  const harness = renderWithProviders(
    <AppShell title="Tracks">
      <div />
    </AppShell>,
    { route: "/tracks" }
  );
  await harness.user.keyboard("/");
  // The header's search button carries the same label, so this picks the
  // palette's own field.
  const input = await screen.findByPlaceholderText(/Search tracks, albums/);
  await harness.user.type(input, "god");
  return harness;
};

describe("the palette with both libraries", () => {
  /**
   * Federated search merges two libraries into one list, so a row has to say
   * which one it came from — otherwise the two "God Is Dead?"s are
   * indistinguishable.
   */
  it("labels each result with its library", async () => {
    server.use(connected(), federated());
    await openPalette();

    const remote = (await screen.findByText("God Is Dead?")).closest("li")!;
    expect(within(remote).getByText("Living room NAS")).toBeInTheDocument();

    const local = (await screen.findByText("God Is Dead? (local)")).closest(
      "li"
    )!;
    expect(within(local).getByText("this machine")).toBeInTheDocument();
  });

  /** With nothing connected there is only one library, so the label is noise. */
  it("says nothing about origin when only the local library exists", async () => {
    server.use(federated());
    await openPalette();

    await screen.findByText("God Is Dead?");
    expect(screen.queryByText("this machine")).toBeNull();
  });

  /**
   * A playlist belongs to one library. With a server connected its playlists
   * are on offer, so a *local* result has none it could join — saving one
   * would produce a row that cannot be played.
   */
  it("offers playlists only to results from the same library", async () => {
    server.use(connected(), federated());
    const { user } = await openPalette();

    const remote = (await screen.findByText("God Is Dead?")).closest("li")!;
    await user.click(
      within(remote).getByRole("button", {
        name: "More actions for God Is Dead?",
      })
    );
    expect(await screen.findByText("Add to playlist")).toBeInTheDocument();
    await user.keyboard("{Escape}");

    const local = (await screen.findByText("God Is Dead? (local)")).closest(
      "li"
    )!;
    await user.click(
      within(local).getByRole("button", {
        name: "More actions for God Is Dead? (local)",
      })
    );
    await waitFor(() =>
      expect(screen.queryByText("Add to playlist")).toBeNull()
    );
  });
});
