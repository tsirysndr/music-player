import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { extensions } from "../../test/handlers";
import { server } from "../../test/server";
import ExtensionsWithData from "./ExtensionsWithData";

describe("ExtensionsWithData", () => {
  it("lists what the daemon reports", async () => {
    renderWithProviders(<ExtensionsWithData />);
    expect(await screen.findByText("Lyrics Provider")).toBeInTheDocument();
    expect(screen.getByText("Mood Predicate")).toBeInTheDocument();
  });

  /** Searching extensions is the palette's job; this page asks for the lot. */
  it("asks the daemon for every extension, unfiltered", async () => {
    const seen = vi.fn();
    server.use(
      graphql.query("GetExtensions", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { extensions } });
      })
    );

    renderWithProviders(<ExtensionsWithData />);
    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.not.objectContaining({ filter: expect.anything() })
      )
    );
  });

  it("says so when none is installed", async () => {
    server.use(
      graphql.query("GetExtensions", () =>
        HttpResponse.json({ data: { extensions: [] } })
      )
    );

    renderWithProviders(<ExtensionsWithData />);
    expect(
      await screen.findByText("No extensions installed")
    ).toBeInTheDocument();
  });

  it("filters by status without another round trip", async () => {
    const seen = vi.fn();
    server.use(
      graphql.query("GetExtensions", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { extensions } });
      })
    );

    const { user } = renderWithProviders(<ExtensionsWithData />);
    await screen.findByText("Lyrics Provider");
    const requests = seen.mock.calls.length;

    // "Mood Predicate" is the disabled one in the fixture.
    await user.click(screen.getByRole("button", { name: "Disabled" }));

    expect(screen.getByText("Mood Predicate")).toBeInTheDocument();
    expect(screen.queryByText("Lyrics Provider")).toBeNull();
    // The response already carries the flag, so nothing was re-fetched.
    expect(seen.mock.calls.length).toBe(requests);
  });

  it("switches an extension off", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("SetExtensionEnabled", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({
          data: { setExtensionEnabled: { ...extensions[0], status: "disabled" } },
        });
      })
    );

    const { user } = renderWithProviders(<ExtensionsWithData />);
    await screen.findByText("Lyrics Provider");

    await user.click(
      screen.getByRole("switch", { name: "Enable Lyrics Provider" })
    );

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith({
        id: "fm.atradio.lyrics-provider",
        enabled: false,
      })
    );
  });

  it("switches a disabled extension back on", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("SetExtensionEnabled", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({
          data: { setExtensionEnabled: { ...extensions[1], status: "enabled" } },
        });
      })
    );

    const { user } = renderWithProviders(<ExtensionsWithData />);
    await screen.findByText("Mood Predicate");

    await user.click(
      screen.getByRole("switch", { name: "Enable Mood Predicate" })
    );

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith({
        id: "com.example.mood",
        enabled: true,
      })
    );
  });

  /** The list has to re-read after a toggle, or the switch springs back. */
  it("re-reads the list after a toggle", async () => {
    let requests = 0;
    server.use(
      graphql.query("GetExtensions", () => {
        requests += 1;
        return HttpResponse.json({ data: { extensions } });
      }),
      graphql.mutation("SetExtensionEnabled", () =>
        HttpResponse.json({
          data: { setExtensionEnabled: { ...extensions[0], status: "disabled" } },
        })
      )
    );

    const { user } = renderWithProviders(<ExtensionsWithData />);
    await screen.findByText("Lyrics Provider");
    const before = requests;

    await user.click(
      screen.getByRole("switch", { name: "Enable Lyrics Provider" })
    );
    await waitFor(() => expect(requests).toBeGreaterThan(before));
  });

  it("rescans on request", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("RescanExtensions", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { rescanExtensions: extensions } });
      })
    );

    const { user } = renderWithProviders(<ExtensionsWithData />);
    await screen.findByText("Lyrics Provider");
    await user.click(screen.getByRole("button", { name: "Rescan extensions" }));

    await waitFor(() => expect(seen).toHaveBeenCalled());
  });

  it("surfaces a failure instead of an empty list", async () => {
    server.use(
      graphql.query("GetExtensions", () =>
        HttpResponse.json({ errors: [{ message: "daemon unreachable" }] })
      )
    );

    renderWithProviders(<ExtensionsWithData />);
    expect(
      await screen.findByText("Unable to load extensions")
    ).toBeInTheDocument();
    expect(screen.getByText("daemon unreachable")).toBeInTheDocument();
  });
});
