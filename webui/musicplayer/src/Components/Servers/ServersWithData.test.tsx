import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import * as fixtures from "../../test/handlers";
import { renderWithProviders, screen, waitFor, within } from "../../test/render";
import { server } from "../../test/server";
import ServersWithData from "./ServersWithData";

const render = () =>
  renderWithProviders(<ServersWithData />, { route: "/servers" });

const row = async (name: string) =>
  (await screen.findByText(name)).closest("div.group\\/row") as HTMLElement;

describe("ServersWithData", () => {
  it("lists the saved servers with their kind and url", async () => {
    render();

    expect(await screen.findByText("Living room NAS")).toBeInTheDocument();
    // The label comes from the registry, not from a table in this file.
    expect(
      screen.getByText("Subsonic / Navidrome · http://192.168.1.10:4533")
    ).toBeInTheDocument();
    expect(
      screen.getByText("Jellyfin · http://media.home.lan:8096")
    ).toBeInTheDocument();
  });

  it("says it is reading locally when nothing is connected", async () => {
    render();
    expect(
      await screen.findByText("Reading from this machine's library.")
    ).toBeInTheDocument();
  });

  /** A dot beside the name, not a filled row. */
  it("marks the connected server and offers the local library back", async () => {
    server.use(
      graphql.query("GetSavedServers", () =>
        HttpResponse.json({
          data: {
            savedServers: fixtures.savedServers.map((entry, index) => ({
              ...entry,
              connected: index === 0,
            })),
          },
        })
      )
    );

    render();
    const connected = await row("Living room NAS");
    expect(within(connected).getByText("Connected")).toBeInTheDocument();
    // The other row still offers to connect.
    const other = await row("Media");
    expect(
      within(other).getByRole("button", { name: "Connect to Media" })
    ).toBeInTheDocument();

    expect(
      screen.getByText("Reading from Living room NAS.")
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Use local library" })
    ).toBeInTheDocument();
  });

  it("connects when a row is clicked", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("ConnectToServer", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({
          data: {
            connectToServer: { ...fixtures.savedServers[0], connected: true },
          },
        });
      })
    );

    const { user } = render();
    await user.click(
      await screen.findByRole("button", { name: "Connect to Living room NAS" })
    );

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith({ id: fixtures.savedServers[0].id })
    );
  });

  /**
   * The daemon's own words, not a generic failure: a wrong password and an
   * unreachable host need different fixes.
   */
  it("shows why a connection was refused", async () => {
    server.use(
      graphql.mutation("ConnectToServer", () =>
        HttpResponse.json({
          errors: [{ message: "authentication failed: wrong username" }],
        })
      )
    );

    const { user } = render();
    await user.click(
      await screen.findByRole("button", { name: "Connect to Media" })
    );

    expect(
      await screen.findByText(/authentication failed: wrong username/)
    ).toBeInTheDocument();
  });

  it("forgets a server", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("DeleteServer", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { deleteServer: true } });
      })
    );

    const { user } = render();
    await user.click(
      await screen.findByRole("button", { name: "Forget Media" })
    );

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith({ id: fixtures.savedServers[1].id })
    );
  });

  describe("the add-server form", () => {
    const open = async () => {
      const harness = render();
      await harness.user.click(
        await screen.findByRole("button", { name: /Add server/ })
      );
      return harness;
    };

    it("offers the kinds the daemon actually supports", async () => {
      await open();
      const select = await screen.findByLabelText("TYPE");
      expect(
        within(select).getByRole("option", { name: "Subsonic / Navidrome" })
      ).toBeInTheDocument();
      expect(
        within(select).getByRole("option", { name: "Jellyfin" })
      ).toBeInTheDocument();
    });

    /** The port each kind listens on, so the example is one that would work. */
    it("shows a url placeholder for the selected kind", async () => {
      const { user } = await open();
      expect(await screen.findByPlaceholderText("http://192.168.1.10:4533"))
        .toBeInTheDocument();

      await user.selectOptions(screen.getByLabelText("TYPE"), "jellyfin");
      expect(
        await screen.findByPlaceholderText("http://192.168.1.10:8096")
      ).toBeInTheDocument();
    });

    /** A peer daemon has no login, so the fields would ask for nothing. */
    it("hides the credentials for a kind that has none", async () => {
      const { user } = await open();
      expect(await screen.findByLabelText("USERNAME")).toBeInTheDocument();

      await user.selectOptions(screen.getByLabelText("TYPE"), "music-player");
      await waitFor(() =>
        expect(screen.queryByLabelText("USERNAME")).toBeNull()
      );
    });

    it("refuses a url with no scheme", async () => {
      const { user } = await open();
      await user.type(await screen.findByLabelText("NAME"), "NAS");
      await user.type(screen.getByLabelText("SERVER URL"), "192.168.1.10:4533");
      await user.click(screen.getByRole("button", { name: "Add server" }));

      expect(await screen.findByText(/Include the scheme/)).toBeInTheDocument();
    });

    /** Adding a server is only ever a step towards using it. */
    it("saves and connects in one go", async () => {
      const added = vi.fn();
      const connected = vi.fn();
      server.use(
        graphql.mutation("AddServer", ({ variables }) => {
          added(variables);
          return HttpResponse.json({
            data: { addServer: { ...fixtures.savedServers[0], id: "new" } },
          });
        }),
        graphql.mutation("ConnectToServer", ({ variables }) => {
          connected(variables);
          return HttpResponse.json({
            data: {
              connectToServer: { ...fixtures.savedServers[0], connected: true },
            },
          });
        })
      );

      const { user } = await open();
      await user.type(await screen.findByLabelText("NAME"), "NAS");
      await user.type(
        screen.getByLabelText("SERVER URL"),
        "http://192.168.1.10:4533"
      );
      await user.click(screen.getByRole("button", { name: "Add server" }));

      await waitFor(() => expect(added).toHaveBeenCalled());
      expect(added.mock.calls[0][0].input).toEqual(
        expect.objectContaining({
          kind: "subsonic",
          name: "NAS",
          url: "http://192.168.1.10:4533",
        })
      );
      await waitFor(() =>
        expect(connected).toHaveBeenCalledWith({ id: "new" })
      );
    });
  });
});
