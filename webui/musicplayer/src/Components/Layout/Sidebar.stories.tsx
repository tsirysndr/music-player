import type { Meta, StoryObj } from "@storybook/react-vite";
import { Provider, createStore } from "jotai";
import { serverConnectedAtom } from "../../State";
import { noop } from "../../Stories/fixtures";
import Sidebar from "./Sidebar";

/**
 * The desktop client's sidebar: wordmark, library sections, recent playlists,
 * the skin switcher and the connection status.
 *
 * The sections come from `Layout/navigation.ts` and are shared with the phone
 * tab bar, so adding one shows up in both.
 *
 * It is hidden below `lg` — widen the canvas to see it.
 */
const meta: Meta<typeof Sidebar> = {
  title: "Layout/Sidebar",
  component: Sidebar,
  args: { onOpenDevices: noop },
  decorators: [
    (Story) => (
      <div className="flex h-dvh bg-window">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof Sidebar>;

export const Default: Story = {};

/**
 * The status dot goes red only when the daemon is unreachable — not when
 * playback is simply on this device, which is the normal case.
 */
export const Disconnected: Story = {
  decorators: [
    (Story) => {
      const store = createStore();
      store.set(serverConnectedAtom, false);
      return (
        <Provider store={store}>
          <Story />
        </Provider>
      );
    },
  ],
};
