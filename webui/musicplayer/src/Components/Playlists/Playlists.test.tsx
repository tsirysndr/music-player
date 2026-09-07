import { describe, expect, it, vi } from "vitest";
import { renderWithProviders, screen, waitFor, within } from "../../test/render";
import Playlists from "./Playlists";

const PLAYLISTS = [
  {
    id: "pl-late-night",
    name: "Late night",
    description: "For the small hours",
    trackCount: 42,
  },
  { id: "pl-focus", name: "Focus", trackCount: 1 },
];

const FOLDERS = [{ id: "folder-moods", name: "Moods" }];

const setup = (props: Partial<React.ComponentProps<typeof Playlists>> = {}) =>
  renderWithProviders(
    <Playlists
      playlists={PLAYLISTS}
      folders={FOLDERS}
      onCreatePlaylist={vi.fn()}
      onCreateFolder={vi.fn()}
      onEditPlaylist={vi.fn()}
      onDeletePlaylist={vi.fn()}
      onPlayPlaylist={vi.fn()}
      {...props}
    />
  );

describe("Playlists", () => {
  it("lists the playlists with their track counts", () => {
    setup();
    expect(screen.getByText("Late night")).toBeInTheDocument();
    expect(screen.getByText("For the small hours · 42 tracks")).toBeInTheDocument();
    // A count of one is singular.
    expect(screen.getByText("1 track")).toBeInTheDocument();
  });

  it("links the folders", () => {
    setup();
    expect(screen.getByRole("link", { name: /Moods/ })).toHaveAttribute(
      "href",
      "/folders/folder-moods"
    );
  });

  /** Searching playlists is the command palette's job now. */
  it("has no filter box of its own", () => {
    setup();
    expect(
      screen.queryByRole("textbox", { name: "Filter playlists…" })
    ).toBeNull();
  });

  it("plays a playlist from its row", async () => {
    const onPlayPlaylist = vi.fn();
    const { user } = setup({ onPlayPlaylist });

    await user.click(screen.getByRole("button", { name: "Play Late night" }));
    expect(onPlayPlaylist).toHaveBeenCalledWith("pl-late-night");
  });

  it("offers to create one when there are none", () => {
    setup({ playlists: [], folders: [] });
    expect(
      screen.getByText("No playlists yet — create one!")
    ).toBeInTheDocument();
  });

  describe("the new-playlist form", () => {
    it("refuses to submit without a name", async () => {
      const onCreatePlaylist = vi.fn();
      const { user } = setup({ onCreatePlaylist });

      await user.click(
        screen.getAllByRole("button", { name: "New playlist" })[0]
      );
      await user.click(
        await screen.findByRole("button", { name: "Create playlist" })
      );

      expect(
        await screen.findByText("Give your playlist a name")
      ).toBeInTheDocument();
      expect(onCreatePlaylist).not.toHaveBeenCalled();
    });

    it("creates a plain playlist", async () => {
      const onCreatePlaylist = vi.fn();
      const { user } = setup({ onCreatePlaylist });

      await user.click(
        screen.getAllByRole("button", { name: "New playlist" })[0]
      );
      const dialog = await screen.findByRole("dialog");

      await user.type(within(dialog).getByLabelText("NAME"), "Deep cuts");
      await user.type(
        within(dialog).getByLabelText("DESCRIPTION"),
        "Album tracks only"
      );
      await user.click(
        within(dialog).getByRole("button", { name: "Create playlist" })
      );

      await waitFor(() =>
        expect(onCreatePlaylist).toHaveBeenCalledWith(
          "Deep cuts",
          "Album tracks only",
          undefined
        )
      );
    });

    /** The name is trimmed, so " Deep cuts " does not become the stored name. */
    it("trims the name", async () => {
      const onCreatePlaylist = vi.fn();
      const { user } = setup({ onCreatePlaylist });

      await user.click(
        screen.getAllByRole("button", { name: "New playlist" })[0]
      );
      const dialog = await screen.findByRole("dialog");

      await user.type(within(dialog).getByLabelText("NAME"), "  Deep cuts  ");
      await user.click(
        within(dialog).getByRole("button", { name: "Create playlist" })
      );

      await waitFor(() =>
        expect(onCreatePlaylist).toHaveBeenCalledWith(
          "Deep cuts",
          expect.anything(),
          undefined
        )
      );
    });

    /** The smart fields only exist once the playlist is a smart one. */
    it("reveals the filter fields when made smart", async () => {
      const { user } = setup();

      await user.click(
        screen.getAllByRole("button", { name: "New playlist" })[0]
      );
      const dialog = await screen.findByRole("dialog");

      expect(within(dialog).queryByLabelText("SORT BY")).toBeNull();

      await user.click(
        within(dialog).getByRole("switch", { name: "Smart playlist" })
      );

      expect(within(dialog).getByLabelText("SORT BY")).toBeInTheDocument();
      expect(within(dialog).getByLabelText("LIMIT")).toBeInTheDocument();
    });

    it("rejects a limit that is not a number", async () => {
      const onCreatePlaylist = vi.fn();
      const { user } = setup({ onCreatePlaylist });

      await user.click(
        screen.getAllByRole("button", { name: "New playlist" })[0]
      );
      const dialog = await screen.findByRole("dialog");

      await user.type(within(dialog).getByLabelText("NAME"), "Rediscover");
      await user.click(
        within(dialog).getByRole("switch", { name: "Smart playlist" })
      );
      await user.type(within(dialog).getByLabelText("LIMIT"), "lots");
      await user.click(
        within(dialog).getByRole("button", { name: "Create playlist" })
      );

      expect(await screen.findByText("Must be a number")).toBeInTheDocument();
      expect(onCreatePlaylist).not.toHaveBeenCalled();
    });

    it("rejects a limit larger than a playlist should hold", async () => {
      const { user } = setup();

      await user.click(
        screen.getAllByRole("button", { name: "New playlist" })[0]
      );
      const dialog = await screen.findByRole("dialog");

      await user.type(within(dialog).getByLabelText("NAME"), "Everything");
      await user.click(
        within(dialog).getByRole("switch", { name: "Smart playlist" })
      );
      await user.type(within(dialog).getByLabelText("LIMIT"), "99999");
      await user.click(
        within(dialog).getByRole("button", { name: "Create playlist" })
      );

      expect(
        await screen.findByText(
          "That is more tracks than a playlist should hold"
        )
      ).toBeInTheDocument();
    });
  });

  describe("the delete confirmation", () => {
    it("names what will be deleted and only deletes once confirmed", async () => {
      const onDeletePlaylist = vi.fn();
      const { user } = setup({ onDeletePlaylist });

      await user.click(
        screen.getByRole("button", { name: "Delete Late night" })
      );

      const dialog = await screen.findByRole("dialog");
      expect(
        within(dialog).getByText(/“Late night” will be removed/)
      ).toBeInTheDocument();
      expect(onDeletePlaylist).not.toHaveBeenCalled();

      await user.click(within(dialog).getByRole("button", { name: "Delete" }));
      expect(onDeletePlaylist).toHaveBeenCalledWith("pl-late-night");
    });

    it("does not delete when cancelled", async () => {
      const onDeletePlaylist = vi.fn();
      const { user } = setup({ onDeletePlaylist });

      await user.click(
        screen.getByRole("button", { name: "Delete Focus" })
      );
      const dialog = await screen.findByRole("dialog");
      await user.click(within(dialog).getByRole("button", { name: "Cancel" }));

      expect(onDeletePlaylist).not.toHaveBeenCalled();
    });
  });

  describe("the edit form", () => {
    it("opens filled in with the playlist", async () => {
      const { user } = setup();

      await user.click(screen.getByRole("button", { name: "Edit Late night" }));
      const dialog = await screen.findByRole("dialog");

      expect(within(dialog).getByLabelText("NAME")).toHaveValue("Late night");
      expect(within(dialog).getByLabelText("DESCRIPTION")).toHaveValue(
        "For the small hours"
      );
    });

    it("saves the edited name", async () => {
      const onEditPlaylist = vi.fn();
      const { user } = setup({ onEditPlaylist });

      await user.click(screen.getByRole("button", { name: "Edit Focus" }));
      const dialog = await screen.findByRole("dialog");

      await user.clear(within(dialog).getByLabelText("NAME"));
      await user.type(within(dialog).getByLabelText("NAME"), "Deep focus");
      await user.click(within(dialog).getByRole("button", { name: "Save" }));

      await waitFor(() =>
        expect(onEditPlaylist).toHaveBeenCalledWith(
          "pl-focus",
          "Deep focus",
          expect.anything()
        )
      );
    });

    it("refuses to save an empty name", async () => {
      const onEditPlaylist = vi.fn();
      const { user } = setup({ onEditPlaylist });

      await user.click(screen.getByRole("button", { name: "Edit Focus" }));
      const dialog = await screen.findByRole("dialog");

      await user.clear(within(dialog).getByLabelText("NAME"));
      await user.click(within(dialog).getByRole("button", { name: "Save" }));

      expect(
        await screen.findByText("Give your playlist a name")
      ).toBeInTheDocument();
      expect(onEditPlaylist).not.toHaveBeenCalled();
    });
  });

  describe("the new-folder form", () => {
    it("creates a folder", async () => {
      const onCreateFolder = vi.fn();
      const { user } = setup({ onCreateFolder });

      await user.click(screen.getByRole("button", { name: "New folder" }));
      const dialog = await screen.findByRole("dialog");

      await user.type(within(dialog).getByLabelText("NAME"), "Workouts");
      await user.click(
        within(dialog).getByRole("button", { name: "Create folder" })
      );

      await waitFor(() =>
        expect(onCreateFolder).toHaveBeenCalledWith("Workouts")
      );
    });

    it("refuses an empty name", async () => {
      const onCreateFolder = vi.fn();
      const { user } = setup({ onCreateFolder });

      await user.click(screen.getByRole("button", { name: "New folder" }));
      await user.click(
        await screen.findByRole("button", { name: "Create folder" })
      );

      expect(
        await screen.findByText("Give your folder a name")
      ).toBeInTheDocument();
      expect(onCreateFolder).not.toHaveBeenCalled();
    });
  });
});
