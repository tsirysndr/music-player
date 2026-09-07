import type { Meta, StoryObj } from "@storybook/react-vite";
import BottomTabs from "./BottomTabs";

/**
 * The phone navigation: the four most-used sections as tabs, everything else
 * behind "More" — which also carries the skin switcher the sidebar owns on a
 * wide screen.
 *
 * It is hidden at `lg` and above, so narrow the canvas to see it.
 */
const meta: Meta<typeof BottomTabs> = {
  title: "Layout/BottomTabs",
  component: BottomTabs,
  decorators: [
    (Story) => (
      <div className="h-dvh bg-window">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof BottomTabs>;

export const Default: Story = {};
