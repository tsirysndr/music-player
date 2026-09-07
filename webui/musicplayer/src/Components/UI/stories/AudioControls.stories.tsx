import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { noop } from "../../../Stories/fixtures";
import { EqBandSlider, SectionHeader, SettingRow } from "..";

/**
 * The two controls the audio-settings dialog is built from, on their own.
 *
 * Both speak the firmware's units: gains are dB × 10, so nothing has to
 * round-trip a float on its way to the engine.
 */
const meta: Meta = {
  title: "UI/Audio controls",
  decorators: [
    (Story) => (
      <div className="w-full bg-window p-6">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj;

const BANDS = [
  { hz: "32", gain: 60 },
  { hz: "64", gain: 40 },
  { hz: "125", gain: 10 },
  { hz: "250", gain: 0 },
  { hz: "500", gain: -20 },
  { hz: "1k", gain: -30 },
  { hz: "2k", gain: 0 },
  { hz: "4k", gain: 25 },
  { hz: "8k", gain: 45 },
  { hz: "16k", gain: 55 },
];

/** A full ten-band curve, the shape the dialog shows. */
export const EqBands: Story = {
  name: "EqBandSlider",
  render: () => {
    const [gains, setGains] = useState(BANDS.map((band) => band.gain));
    return (
      <div className="flex h-[150px] gap-[2px]">
        {BANDS.map((band, index) => (
          <EqBandSlider
            key={band.hz}
            gain={gains[index]}
            freqLabel={band.hz}
            onChange={(gain) =>
              setGains((current) =>
                current.map((value, at) => (at === index ? gain : value))
              )
            }
          />
        ))}
      </div>
    );
  },
};

/** Bypassed: the curve stays readable, it just dims. */
export const EqBandsDisabled: Story = {
  render: () => (
    <div className="flex h-[150px] gap-[2px] opacity-[0.35]">
      {BANDS.map((band) => (
        <EqBandSlider
          key={band.hz}
          gain={band.gain}
          freqLabel={band.hz}
          disabled
          onChange={noop}
        />
      ))}
    </div>
  ),
};

/** Every band flat — where a double-click on one puts it. */
export const EqBandsFlat: Story = {
  render: () => (
    <div className="flex h-[150px] gap-[2px]">
      {BANDS.map((band) => (
        <EqBandSlider
          key={band.hz}
          gain={0}
          freqLabel={band.hz}
          onChange={noop}
        />
      ))}
    </div>
  ),
};

export const SettingRows: Story = {
  name: "SettingRow",
  render: () => {
    const [preamp, setPreamp] = useState(0.5);
    return (
      <div className="flex max-w-[420px] flex-col gap-2">
        <SectionHeader title="LEVELS" className="mb-1" />
        <SettingRow
          label="Pre-amp"
          valueText={`${((preamp * 240 - 120) / 10).toFixed(1)} dB`}
          progress={preamp}
          onChange={setPreamp}
        />
        <SettingRow
          label="Fade in"
          valueText="2 s"
          progress={2 / 15}
          onChange={noop}
        />
        <SettingRow
          label="Fade out"
          valueText="0 s"
          progress={0}
          disabled
          onChange={noop}
        />
        <SettingRow
          label="A label long enough to be clipped"
          valueText="100%"
          progress={1}
          onChange={noop}
        />
      </div>
    );
  },
};
