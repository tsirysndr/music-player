import { describe, expect, it } from "vitest";
import { renderWithProviders, screen, within } from "../../test/render";
import PlaylistsWithData from "./PlaylistsWithData";

describe("PlaylistsWithData", () => {
  /**
   * A listing reports a count without sending the entries, so anything that
   * counts `tracks` client-side gets zero. Both clients did, and every row
   * read "0 tracks".
   */
  it("shows each playlist's track count", async () => {
    renderWithProviders(<PlaylistsWithData />, { route: "/playlists" });

    // The sidebar lists recents too, so this looks inside the page.
    const main = await screen.findByRole("main");
    expect(await within(main).findByText(/42 tracks/)).toBeInTheDocument();
    // Singular, not "1 tracks".
    expect(within(main).getByText(/\b1 track\b/)).toBeInTheDocument();
  });
});
