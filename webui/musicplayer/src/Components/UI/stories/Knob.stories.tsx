import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import Knob from "../Knob";

/**
 * The Mixxx-style rotary from the desktop player bar. Drag up and down or
 * scroll to turn it; double-click resets to the default.
 */
const meta: Meta<typeof Knob> = {
  title: "Design System/Knob",
  component: Knob,
  decorators: [
    (Story) => (
      <div className="p-8">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof Knob>;

/** Live: drag it, scroll it, double-click to reset. */
export const Interactive: Story = {
  render: function Interactive() {
    const [value, setValue] = useState(0.75);
    return (
      <Knob
        norm={value}
        valueText={`${Math.round(value * 100)}%`}
        label="Volume"
        defaultNorm={0.75}
        onChange={setValue}
      />
    );
  },
};

export const Sizes: Story = {
  render: () => (
    <div className="flex items-end gap-6">
      <Knob size={40} norm={0.3} valueText="30%" onChange={() => {}} />
      <Knob size={54} norm={0.6} valueText="60%" label="Precut" onChange={() => {}} />
      <Knob size={72} norm={0.9} valueText="90%" label="Gain" onChange={() => {}} />
    </div>
  ),
};

/** Without a label — how the player bar draws the volume knob. */
export const Unlabelled: Story = {
  args: { norm: 0.5, valueText: "50%", onChange: () => {} },
};
