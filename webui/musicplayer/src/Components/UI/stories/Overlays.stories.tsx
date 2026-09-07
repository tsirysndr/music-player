import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { noop, recentPlaylists } from "../../../Stories/fixtures";
import {
  Button,
  ContextMenu,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  Dialog,
  IconButton,
  Icons,
} from "..";

/**
 * The two overlays every page hangs off: the modal shell and the "…" menu.
 *
 * Both are open by default here — a story of a closed overlay is a blank
 * canvas.
 */
const meta: Meta = {
  title: "UI/Overlays",
  decorators: [
    (Story) => (
      <div className="flex min-h-dvh items-center justify-center bg-window p-6">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj;

export const DialogDefault: Story = {
  name: "Dialog",
  render: () => (
    <Dialog
      isOpen
      onClose={noop}
      title="Rename playlist"
      icon={Icons.playlist}
      footer={
        <>
          <Button variant="outline" onClick={noop}>
            Cancel
          </Button>
          <Button onClick={noop}>Save</Button>
        </>
      }
    >
      <p className="py-2 text-[13px] text-dim">
        The panel, the hairline border, the title row and the “esc to close”
        hint are shared by every overlay in the app.
      </p>
    </Dialog>
  ),
};

export const DialogDestructive: Story = {
  render: () => (
    <Dialog
      isOpen
      onClose={noop}
      title="Delete playlist"
      icon={Icons.trash}
      width={420}
      // A destructive confirmation does not dismiss on a stray backdrop click.
      isDismissable={false}
      footer={
        <>
          <Button variant="outline" onClick={noop}>
            Cancel
          </Button>
          <Button variant="danger" onClick={noop}>
            Delete
          </Button>
        </>
      }
    >
      <p className="py-2 text-[13px] text-dim">
        “Late night” and its 42 tracks will be removed. The files stay where
        they are.
      </p>
    </Dialog>
  ),
};

/** Wide, scrolling body — the shape the audio settings use. */
export const DialogTall: Story = {
  render: () => (
    <Dialog isOpen onClose={noop} title="A long one" icon={Icons.settings} width={780}>
      <div className="flex flex-col gap-3 py-2">
        {Array.from({ length: 30 }, (_, index) => (
          <p key={index} className="text-[13px] text-dim">
            Row {index + 1}
          </p>
        ))}
      </div>
    </Dialog>
  ),
};

/** Sat near the top rather than centred — how the command palette opens. */
export const DialogTopPlacement: Story = {
  render: () => (
    <Dialog isOpen onClose={noop} title="Near the top" placement="top" width={620}>
      <p className="py-2 text-[13px] text-dim">
        A palette wants to be under the cursor, not in the middle of the screen.
      </p>
    </Dialog>
  ),
};

export const ContextMenuDefault: Story = {
  name: "ContextMenu",
  render: () => {
    const Trigger = <IconButton icon={Icons.ellipsis} aria-label="Track menu" />;
    return (
      <ContextMenu trigger={Trigger} placement="bottom start">
        <ContextMenuItem icon={Icons.music} label="Play next" onClick={noop} />
        <ContextMenuItem
          icon={Icons.listMusic}
          label="Add to queue"
          onClick={noop}
        />
        <ContextMenuSeparator />
        <ContextMenuLabel>ADD TO PLAYLIST</ContextMenuLabel>
        {recentPlaylists.map((playlist) => (
          <ContextMenuItem
            key={playlist.id}
            icon={Icons.playlist}
            label={playlist.name}
            onClick={noop}
          />
        ))}
        <ContextMenuSeparator />
        <ContextMenuItem icon={Icons.trash} label="Remove" danger onClick={noop} />
      </ContextMenu>
    );
  },
};

/** Two entries — the case the old 6px panel padding made look hollow. */
export const ContextMenuShort: Story = {
  render: () => (
    <ContextMenu
      trigger={<IconButton icon={Icons.ellipsis} aria-label="Album menu" />}
      width={180}
    >
      <ContextMenuItem icon={Icons.pencil} label="Edit" onClick={noop} />
      <ContextMenuItem icon={Icons.trash} label="Delete" danger onClick={noop} />
    </ContextMenu>
  ),
};

/** Both together, driven from a button, as a page actually uses them. */
export const Interactive: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    return (
      <div className="flex items-center gap-3">
        <Button onClick={() => setOpen(true)}>Open dialog</Button>
        <ContextMenu
          trigger={<IconButton icon={Icons.ellipsis} aria-label="Menu" />}
        >
          <ContextMenuItem icon={Icons.music} label="Play next" onClick={noop} />
          <ContextMenuItem icon={Icons.disc} label="Go to album" onClick={noop} />
        </ContextMenu>
        <Dialog
          isOpen={open}
          onClose={() => setOpen(false)}
          title="Hello"
          icon={Icons.disc}
        >
          <p className="py-2 text-[13px] text-dim">Escape closes it.</p>
        </Dialog>
      </div>
    );
  },
};
