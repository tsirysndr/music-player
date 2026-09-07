import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { noop } from "../../Stories/fixtures";
import AudioSettings, { type AudioSettingsState } from "./AudioSettings";

/** The band centre frequencies the engine uses, 32 Hz … 16 kHz. */
const BAND_FREQUENCIES = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

const bands = (gains: number[] = []) =>
  BAND_FREQUENCIES.map((cutoff, i) => ({
    cutoff,
    q: 10,
    gain: gains[i] ?? 0,
  }));

const FLAT: AudioSettingsState = {
  eqEnabled: true,
  eqPrecut: 0,
  eqBands: bands(),
  bass: 0,
  bassMin: -24,
  bassMax: 24,
  treble: 0,
  trebleMin: -24,
  trebleMax: 24,
  balance: 0,
  replaygainType: 3,
  replaygainPreamp: 0,
  replaygainNoclip: false,
  crossfade: 0,
  fadeInDelay: 0,
  fadeInDuration: 2,
  fadeOutDelay: 0,
  fadeOutDuration: 2,
  fadeOutMixmode: 0,
  dithering: false,
};

/**
 * The audio-settings modal — the desktop's `show-audio` overlay, opened with
 * `e` or from the equalizer button in the player bar.
 *
 * Everything is in the firmware's own units (dB × 10 where a fraction is
 * meaningful), and every control writes straight through: the daemon clamps,
 * persists and hands the whole state back.
 */
const meta: Meta<typeof AudioSettings> = {
  title: "Layout/AudioSettings",
  component: AudioSettings,
  parameters: { layout: "fullscreen" },
  args: {
    isOpen: true,
    settings: FLAT,
    onClose: noop,
    onSet: noop,
    onSetEqBand: noop,
  },
};

export default meta;

type Story = StoryObj<typeof AudioSettings>;

export const Flat: Story = {};

/** A smiley curve, the shape people actually dial in. */
export const SmileyCurve: Story = {
  args: {
    settings: {
      ...FLAT,
      eqPrecut: 60,
      eqBands: bands([80, 60, 30, 0, -30, -40, -20, 20, 60, 90]),
      bass: 4,
      treble: 3,
    },
  },
};

/** EQ off: the bands dim and stop taking input, the precut knob stays. */
export const EqDisabled: Story = {
  args: { settings: { ...FLAT, eqEnabled: false } },
};

export const ReplayGainOn: Story = {
  args: {
    settings: {
      ...FLAT,
      replaygainType: 1,
      replaygainPreamp: -30,
      replaygainNoclip: true,
    },
  },
};

/** Crossfade off leaves every fade row inert — there is nothing to fade. */
export const CrossfadeOff: Story = {
  args: { settings: { ...FLAT, crossfade: 0 } },
};

export const CrossfadeAlways: Story = {
  args: {
    settings: {
      ...FLAT,
      crossfade: 5,
      fadeInDelay: 1,
      fadeInDuration: 6,
      fadeOutDelay: 2,
      fadeOutDuration: 8,
      fadeOutMixmode: 2,
    },
  },
};

export const BalanceLeft: Story = {
  args: { settings: { ...FLAT, balance: -40 } },
};

export const Loading: Story = {
  args: { settings: undefined, loading: true },
};

export const Failed: Story = {
  args: { settings: undefined, error: "daemon unreachable" },
};

/** Live: drag the bands, turn the knobs, change the modes. */
export const Interactive: Story = {
  render: function Interactive(args) {
    const [state, setState] = useState<AudioSettingsState>({
      ...FLAT,
      eqPrecut: 30,
    });

    // The daemon clamps and hands the whole state back; this stands in for it.
    const set = (name: string, value: number) =>
      setState((current) => {
        const next = { ...current };
        const map: Record<string, (v: number) => void> = {
          eq_enabled: (v) => (next.eqEnabled = v !== 0),
          eq_precut: (v) => (next.eqPrecut = Math.min(240, Math.max(0, v))),
          bass: (v) => (next.bass = Math.min(24, Math.max(-24, v))),
          treble: (v) => (next.treble = Math.min(24, Math.max(-24, v))),
          balance: (v) => (next.balance = Math.min(100, Math.max(-100, v))),
          rg_type: (v) => (next.replaygainType = v),
          rg_preamp: (v) => (next.replaygainPreamp = v),
          rg_noclip: (v) => (next.replaygainNoclip = v !== 0),
          crossfade: (v) => (next.crossfade = v),
          fade_in_delay: (v) => (next.fadeInDelay = v),
          fade_in_duration: (v) => (next.fadeInDuration = v),
          fade_out_delay: (v) => (next.fadeOutDelay = v),
          fade_out_duration: (v) => (next.fadeOutDuration = v),
          fade_out_mixmode: (v) => (next.fadeOutMixmode = v),
          dithering: (v) => (next.dithering = v !== 0),
        };
        map[name]?.(value);
        return next;
      });

    return (
      <AudioSettings
        {...args}
        settings={state}
        onSet={set}
        onSetEqBand={(band, gain) =>
          setState((current) => ({
            ...current,
            eqBands: current.eqBands.map((entry, i) =>
              i === band ? { ...entry, gain } : entry
            ),
          }))
        }
      />
    );
  },
};
