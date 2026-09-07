import type { Meta, StoryObj } from "@storybook/react-vite";
import VfdDisplay from "../VfdDisplay";

/**
 * The jetAudio-style readout from the right of the desktop player bar:
 * transport state, elapsed time in the mono face, two VU strips, and the
 * stream's codec line underneath.
 */
const meta: Meta<typeof VfdDisplay> = {
  title: "Design System/VfdDisplay",
  component: VfdDisplay,
  args: {
    timeText: "02:41",
    infoText: "FLAC  1411 kbps  44.1 kHz",
    playing: true,
    stopped: false,
    vuLeft: 0.62,
    vuRight: 0.55,
  },
};

export default meta;

type Story = StoryObj<typeof VfdDisplay>;

export const Playing: Story = {};

export const Paused: Story = {
  args: { playing: false, vuLeft: 0, vuRight: 0 },
};

/** Stopped is a square — the tape-deck convention the desktop keeps. */
export const Stopped: Story = {
  args: {
    stopped: true,
    playing: false,
    timeText: "00:00",
    infoText: "-- kbps  --.- kHz",
    vuLeft: 0,
    vuRight: 0,
  },
};

/** A live stream has no meaningful position, so it names the source instead. */
export const Radio: Story = {
  args: { timeText: "RADIO", infoText: "internet radio" },
};

export const Peaking: Story = {
  args: { vuLeft: 1, vuRight: 0.95 },
};
