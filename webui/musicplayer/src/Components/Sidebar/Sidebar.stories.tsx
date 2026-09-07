import type { Meta, StoryObj } from "@storybook/react-vite";
import Sidebar from "./Sidebar";

const meta: Meta<typeof Sidebar> = {
  title: "Components/Sidebar",
  component: Sidebar,
};

export default meta;

export const Default: StoryObj<typeof Sidebar> = {
  args: {
  devices: [],
  currentDevice: undefined,
  connectToDevice: (_id: string) => {},
  disconnectFromDevice: () => {},
},
};
