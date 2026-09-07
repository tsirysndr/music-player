import type { Meta, StoryObj } from "@storybook/react-vite";
import { noop } from "../../Stories/fixtures";
import { Button, Icons } from "../UI";
import Header from "./Header";

/**
 * The 62px header: title, optional page actions, the search button, and the
 * two panel toggles. The title comes from the route unless one is given.
 *
 * Search filters never live here — they belong next to the list they filter.
 */
const meta: Meta<typeof Header> = {
  title: "Layout/Header",
  component: Header,
};

export default meta;

type Story = StoryObj<typeof Header>;

export const Default: Story = { args: { title: "Albums" } };

/** A detail page: a back chevron and the record's own name. */
export const WithBack: Story = {
  args: { title: "13 (Deluxe Version)", onBack: noop },
};

export const WithActions: Story = {
  args: {
    title: "Playlists",
    actions: <Button icon={Icons.circlePlus}>New playlist</Button>,
  },
};
