import type { Meta, StoryObj } from "@storybook/react-vite";
import LikeButton from "../LikeButton";

/** The desktop's heart, filled when liked and an outline when not. */
const meta: Meta<typeof LikeButton> = {
  title: "Design System/LikeButton",
  component: LikeButton,
};

export default meta;

type Story = StoryObj<typeof LikeButton>;

export const NotLiked: Story = {};
export const Liked: Story = { args: { liked: true } };
/** Dim until the row is hovered — how a track table draws it. */
export const Subtle: Story = { args: { subtle: true } };
