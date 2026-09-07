import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import Toggle from "../Toggle";

const meta: Meta<typeof Toggle> = {
  title: "Design System/Toggle",
  component: Toggle,
  args: { label: "Enable EQ" },
};

export default meta;

type Story = StoryObj<typeof Toggle>;

export const Off: Story = { args: { checked: false, onChange: () => {} } };
export const On: Story = { args: { checked: true, onChange: () => {} } };
export const Disabled: Story = {
  args: { checked: true, disabled: true, onChange: () => {} },
};

export const Interactive: Story = {
  render: function Interactive() {
    const [on, setOn] = useState(false);
    return <Toggle checked={on} label="Enable EQ" onChange={setOn} />;
  },
};
