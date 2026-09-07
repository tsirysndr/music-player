import type { Meta, StoryObj } from "@storybook/react-vite";
import Select from "../Select";
import TextField, { TextAreaField } from "../TextField";

/**
 * The form fields. Every form in the app is react-hook-form with a zod
 * resolver, so each takes an `error` and renders the message itself.
 */
const meta: Meta<typeof TextField> = {
  title: "Design System/Fields",
  component: TextField,
  decorators: [
    (Story) => (
      <div className="w-80 p-8">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof TextField>;

export const Text: Story = {
  args: { label: "NAME", placeholder: "Give your playlist a title" },
};

export const WithHint: Story = {
  args: { label: "LIMIT", placeholder: "0 = all", hint: "0 means every track" },
};

/** The error replaces the hint — two competing instructions help nobody. */
export const WithError: Story = {
  args: {
    label: "LIMIT",
    placeholder: "0 = all",
    hint: "0 means every track",
    error: "Must be a number",
  },
};

export const Mono: Story = {
  args: { label: "FILTER", mono: true, placeholder: "genre==rock" },
};

export const TextArea: StoryObj<typeof TextAreaField> = {
  render: () => (
    <TextAreaField label="DESCRIPTION" placeholder="Write a description" />
  ),
};

export const Dropdown: StoryObj<typeof Select> = {
  render: () => (
    <div className="flex flex-col gap-4">
      <Select
        label="SORT BY"
        options={[
          { value: "", label: "Library order" },
          { value: "random", label: "Random" },
          { value: "year", label: "Year" },
        ]}
      />
      <Select
        label="ORDER"
        error="Pick one"
        options={[
          { value: "asc", label: "Ascending" },
          { value: "desc", label: "Descending" },
        ]}
      />
    </div>
  ),
};
