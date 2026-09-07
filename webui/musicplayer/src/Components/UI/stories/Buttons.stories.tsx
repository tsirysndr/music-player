import type { Meta, StoryObj } from "@storybook/react-vite";
import Button from "../Button";
import { Icons } from "../icons";

const meta: Meta<typeof Button> = {
  title: "Design System/Button",
  component: Button,
  args: { children: "New playlist" },
};

export default meta;

type Story = StoryObj<typeof Button>;

export const Accent: Story = {};
export const Outline: Story = { args: { variant: "outline" } };
export const Ghost: Story = { args: { variant: "ghost", children: "Cancel" } };
export const Danger: Story = { args: { variant: "danger", children: "Delete" } };
export const WithIcon: Story = { args: { icon: Icons.circlePlus } };
export const Disabled: Story = { args: { disabled: true } };
export const Block: Story = {
  args: { block: true },
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
};

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      <Button>Accent</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="danger">Danger</Button>
      <Button icon={Icons.circlePlus}>With icon</Button>
      <Button disabled>Disabled</Button>
    </div>
  ),
};
