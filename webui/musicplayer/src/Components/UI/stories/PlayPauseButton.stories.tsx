import type { Meta, StoryObj } from "@storybook/react-vite";
import PlayPauseButton from "../PlayPauseButton";

/** The filled accent disc at the centre of the transport. */
const meta: Meta<typeof PlayPauseButton> = {
  title: "Design System/PlayPauseButton",
  component: PlayPauseButton,
};

export default meta;

type Story = StoryObj<typeof PlayPauseButton>;

export const Play: Story = {};
export const Pause: Story = { args: { playing: true } };
export const Sizes: Story = {
  render: () => (
    <div className="flex items-center gap-4">
      <PlayPauseButton size={32} onClick={() => {}} />
      <PlayPauseButton size={44} onClick={() => {}} />
      <PlayPauseButton size={64} playing onClick={() => {}} />
    </div>
  ),
};
