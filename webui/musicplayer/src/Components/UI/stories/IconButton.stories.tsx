import type { Meta, StoryObj } from "@storybook/react-vite";
import IconButton from "../IconButton";
import { Icons } from "../icons";

/**
 * The desktop's `IconButton`: a circular hover target whose glyph goes from
 * dim to full text colour, or sits in the accent colour when what it toggles
 * is on.
 */
const meta: Meta<typeof IconButton> = {
  title: "Design System/IconButton",
  component: IconButton,
  args: { icon: Icons.shuffle, "aria-label": "Shuffle" },
};

export default meta;

type Story = StoryObj<typeof IconButton>;

export const Default: Story = {};
export const Accented: Story = { args: { accented: true } };
export const Disabled: Story = { args: { disabled: true } };

export const Sizes: Story = {
  render: () => (
    <div className="flex items-center gap-3">
      <IconButton icon={Icons.next} iconSize={13} size={26} aria-label="Next" />
      <IconButton icon={Icons.next} iconSize={16} size={34} aria-label="Next" />
      <IconButton icon={Icons.next} iconSize={22} size={48} aria-label="Next" />
    </div>
  ),
};

/** The transport set, as the player bar draws it. */
export const Transport: Story = {
  render: () => (
    <div className="flex items-center gap-2">
      <IconButton icon={Icons.shuffle} iconSize={15} aria-label="Shuffle" />
      <IconButton icon={Icons.prev} iconSize={17} aria-label="Previous" />
      <IconButton icon={Icons.next} iconSize={17} aria-label="Next" />
      <IconButton icon={Icons.repeat} iconSize={15} accented aria-label="Repeat" />
      <IconButton icon={Icons.equalizer} iconSize={16} aria-label="Audio settings" />
      <IconButton icon={Icons.listMusic} iconSize={16} aria-label="Queue" />
    </div>
  ),
};
