import type { Meta, StoryObj } from "@storybook/react-vite";
import Like from "./Like";

const meta: Meta<typeof Like> = {
  title: "Components/Like",
  component: Like,
};

export default meta;

export const Default: StoryObj<typeof Like> = {};
