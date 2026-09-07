import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import SlideBar from "../SlideBar";

/** Click or drag anywhere on the track — the seek bar and the setting rows. */
const meta: Meta<typeof SlideBar> = {
  title: "Design System/SlideBar",
  component: SlideBar,
  decorators: [
    (Story) => (
      <div className="w-96 p-8">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof SlideBar>;

export const Interactive: Story = {
  render: function Interactive() {
    const [value, setValue] = useState(0.4);
    return <SlideBar progress={value} onChange={setValue} />;
  },
};

export const Empty: Story = { args: { progress: 0, onChange: () => {} } };
export const Full: Story = { args: { progress: 1, onChange: () => {} } };
export const Thin: Story = {
  args: { progress: 0.6, barHeight: 4, onChange: () => {} },
};
/** Nothing playing: the bar is inert. */
export const Disabled: Story = {
  args: { progress: 0.3, disabled: true, onChange: () => {} },
};
