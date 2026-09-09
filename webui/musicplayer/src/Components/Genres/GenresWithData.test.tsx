import { describe, expect, it } from "vitest";
import { renderWithProviders, screen, within } from "../../test/render";
import GenresWithData from "./GenresWithData";

const render = () =>
  renderWithProviders(<GenresWithData />, { route: "/genres" });

describe("GenresWithData", () => {
  it("lists the library's genres", async () => {
    render();
    expect(await screen.findByText("Hip Hop")).toBeInTheDocument();
    expect(screen.getByText("Shoegaze")).toBeInTheDocument();
  });

  it("counts tracks, and says one track rather than 1 tracks", async () => {
    render();
    expect(await screen.findByText("42 tracks")).toBeInTheDocument();
    expect(screen.getByText("1 track")).toBeInTheDocument();
  });

  /**
   * A remote server that reports no count has not said the genre is empty, so
   * showing "0 tracks" would be inventing a fact.
   */
  it("says nothing when the count is unknown", async () => {
    render();
    const tile = (await screen.findByText("Ambient")).closest("button")!;
    expect(within(tile).queryByText(/track/)).toBeNull();
  });

  it("filters as you type", async () => {
    const { user } = render();
    await user.type(
      await screen.findByPlaceholderText("Filter genres…"),
      "shoe"
    );
    expect(screen.getByText("Shoegaze")).toBeInTheDocument();
    expect(screen.queryByText("Hip Hop")).toBeNull();
  });
});
