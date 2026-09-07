import type { Meta, StoryObj } from "@storybook/react-vite";
import Search from "./Search";

const meta: Meta<typeof Search> = {
  title: "Components/Search",
  component: Search,
};

export default meta;

export const Default: StoryObj<typeof Search> = {};
