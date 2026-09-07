import type { Meta, StoryObj } from "@storybook/react-vite";
import Library from "./Library";

const meta: Meta<typeof Library> = {
  title: "Components/Library",
  component: Library,
};

export default meta;

export const Default: StoryObj<typeof Library> = {};
