import type { Meta, StoryObj } from "@storybook/react-vite";
import Filter from "./Filter";

const meta: Meta<typeof Filter> = {
  title: "Components/Filter",
  component: Filter,
  argTypes: {
    onChange: { action: "onChange" },
  },
};

export default meta;

export const Default: StoryObj<typeof Filter> = {};
