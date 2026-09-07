import type { Meta, StoryObj } from "@storybook/react-vite";
import MeterStrip from "../MeterStrip";

/**
 * The Mixxx-style segmented LED strip inside the VFD. The top three segments
 * are the "high" colour and the next three "mid", so a peak reads at a glance
 * without a number.
 */
const meta: Meta<typeof MeterStrip> = {
  title: "Design System/MeterStrip",
  component: MeterStrip,
  args: { level: 0.6 },
  argTypes: {
    level: { control: { type: "range", min: 0, max: 1, step: 0.01 } },
    segments: { control: { type: "number" } },
  },
  decorators: [
    (Story) => (
      <div className="w-64 p-6">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof MeterStrip>;

export const Default: Story = {};
export const Silent: Story = { args: { level: 0 } };
export const Peaking: Story = { args: { level: 1 } };
export const Coarse: Story = { args: { level: 0.7, segments: 8 } };
