import {
  Dialog,
  EqBandSlider,
  Icons,
  Knob,
  SectionHeader,
  Select,
  Toggle,
} from "../UI";

/** One EQ band, in the firmware's units. */
export type EqBand = {
  /** Centre frequency in Hz. */
  cutoff: number;
  q: number;
  /** dB × 10. */
  gain: number;
};

/**
 * The DSP chain's state, exactly as the daemon reports it.
 *
 * Everything is an integer in the firmware's own units — dB × 10 wherever a
 * fraction is meaningful — so nothing has to round-trip a float through a
 * slider. The ranges come with the values because a slider needs them and
 * they are not derivable from the field alone.
 */
export type AudioSettingsState = {
  eqEnabled: boolean;
  /** dB × 10, 0..240. */
  eqPrecut: number;
  eqBands: EqBand[];
  bass: number;
  bassMin: number;
  bassMax: number;
  treble: number;
  trebleMin: number;
  trebleMax: number;
  /** -100 (full left)..100 (full right). */
  balance: number;
  /** 0 track, 1 album, 2 track (shuffle), 3 off. */
  replaygainType: number;
  /** dB × 10, -120..120. */
  replaygainPreamp: number;
  replaygainNoclip: boolean;
  /** 0 off … 5 always. */
  crossfade: number;
  fadeInDelay: number;
  fadeInDuration: number;
  fadeOutDelay: number;
  fadeOutDuration: number;
  /** 0 crossfade, 2 mix. */
  fadeOutMixmode: number;
  dithering: boolean;
};

export type AudioSettingsProps = {
  isOpen: boolean;
  settings?: AudioSettingsState;
  loading?: boolean;
  error?: string;
  onClose: () => void;
  /** The daemon's own setting names — `eq_enabled`, `bass`, `crossfade`, … */
  onSet: (name: string, value: number) => void;
  /** Band index and the new gain in dB × 10. */
  onSetEqBand: (band: number, gain: number) => void;
};

// The option order *is* the wire value in both lists — the daemon takes the
// index — so these stay in the desktop's order rather than a friendlier one.
const REPLAYGAIN_MODES = [
  { value: "0", label: "Track" },
  { value: "1", label: "Album" },
  { value: "2", label: "Track (shuffle)" },
  { value: "3", label: "Off" },
];

const CROSSFADE_MODES = [
  { value: "0", label: "Off" },
  { value: "1", label: "Auto track change" },
  { value: "2", label: "Manual track change" },
  { value: "3", label: "Shuffle" },
  { value: "4", label: "Shuffle or manual skip" },
  { value: "5", label: "Always" },
];

const MIXMODES = [
  { value: "0", label: "Crossfade" },
  { value: "2", label: "Mix" },
];

/** `1000` → `1k`, matching the desktop's band labels. */
const freqLabel = (hz: number) => (hz >= 1000 ? `${hz / 1000}k` : `${hz}`);

/**
 * A decibel reading from tenths, e.g. `-7.5 dB` — and `0 dB`, not `0.0 dB`.
 * Slint prints the shortest form and the readouts are only 9px wide, so the
 * trailing zero goes.
 */
const db = (tenths: number) =>
  `${tenths % 10 === 0 ? tenths / 10 : (tenths / 10).toFixed(1)} dB`;

/** Balance as the desktop words it: `center`, `20% L`, `20% R`. */
const balanceLabel = (value: number) => {
  if (value === 0) return "center";
  return value < 0 ? `${-value}% L` : `${value}% R`;
};

/** The 12px dim caption the desktop puts to the left of a control. */
const FieldLabel = ({
  children,
  fixed,
}: {
  children: string;
  fixed?: boolean;
}) => (
  <span
    className={`shrink-0 text-xs text-dim ${fixed ? "w-11" : ""}`}
  >
    {children}
  </span>
);

/**
 * The audio-settings modal — a port of the desktop's `show-audio` overlay,
 * opened with `e` or from the equalizer button in the player bar.
 *
 * Every control writes straight through: the daemon clamps, persists to
 * `settings.toml` and hands the whole state back, so what is on screen is what
 * the engine actually took rather than what was asked for.
 *
 * Presentational — `AudioSettingsWithData` supplies the state.
 */
const AudioSettings = ({
  isOpen,
  settings,
  loading,
  error,
  onClose,
  onSet,
  onSetEqBand,
}: AudioSettingsProps) => (
  <Dialog
    isOpen={isOpen}
    onClose={onClose}
    title="Audio Settings"
    icon={Icons.equalizer}
    width={780}
  >
    {error ? (
      <p className="py-10 text-center text-[13px] text-syntax-error">{error}</p>
    ) : loading || !settings ? (
      <p className="py-10 text-center text-xs text-muted">Loading…</p>
    ) : (
      <div className="flex flex-col gap-[10px] pb-2">
        {/* ── Equalizer ─────────────────────────────────────────────────── */}
        <SectionHeader title="EQUALIZER" />
        <div className="flex h-[26px] items-center gap-[10px]">
          <FieldLabel>Enable EQ</FieldLabel>
          <Toggle
            checked={settings.eqEnabled}
            label="Enable EQ"
            onChange={(next) => onSet("eq_enabled", next ? 1 : 0)}
          />
        </div>

        {/* The bands dim rather than disappear when the EQ is off, so the
            curve you set stays readable while it is bypassed. */}
        <div
          className={`flex h-[150px] items-stretch gap-[2px] transition-opacity ${
            settings.eqEnabled ? "" : "opacity-[0.35]"
          }`}
        >
          <div className="flex min-w-0 flex-1 gap-[2px]">
            {settings.eqBands.map((band, index) => (
              <EqBandSlider
                key={band.cutoff}
                gain={band.gain}
                freqLabel={freqLabel(band.cutoff)}
                disabled={!settings.eqEnabled}
                onChange={(gain) => onSetEqBand(index, gain)}
              />
            ))}
          </div>
          <div className="my-[15px] w-px shrink-0 bg-line" />
          <div className="flex shrink-0 items-center">
            <Knob
              label="Precut"
              // Precut is headroom, so it only goes one way: 0..24 dB.
              valueText={db(settings.eqPrecut)}
              norm={settings.eqPrecut / 240}
              defaultNorm={0}
              onChange={(value) =>
                onSet("eq_precut", Math.round(value * 48) * 5)
              }
            />
          </div>
        </div>

        {/* ── Tone │ ReplayGain ─────────────────────────────────────────── */}
        <div className="flex flex-col gap-[18px] sm:flex-row">
          <div className="flex min-w-0 flex-1 flex-col gap-[10px]">
            <SectionHeader title="TONE" />
            <div className="flex justify-center gap-[22px] pt-[2px]">
              <Knob
                label="Bass"
                valueText={`${settings.bass} dB`}
                norm={
                  (settings.bass - settings.bassMin) /
                  (settings.bassMax - settings.bassMin)
                }
                defaultNorm={
                  (0 - settings.bassMin) / (settings.bassMax - settings.bassMin)
                }
                onChange={(value) =>
                  onSet(
                    "bass",
                    Math.round(
                      settings.bassMin +
                        value * (settings.bassMax - settings.bassMin)
                    )
                  )
                }
              />
              <Knob
                label="Treble"
                valueText={`${settings.treble} dB`}
                norm={
                  (settings.treble - settings.trebleMin) /
                  (settings.trebleMax - settings.trebleMin)
                }
                defaultNorm={
                  (0 - settings.trebleMin) /
                  (settings.trebleMax - settings.trebleMin)
                }
                onChange={(value) =>
                  onSet(
                    "treble",
                    Math.round(
                      settings.trebleMin +
                        value * (settings.trebleMax - settings.trebleMin)
                    )
                  )
                }
              />
              <Knob
                label="Balance"
                valueText={balanceLabel(settings.balance)}
                norm={(settings.balance + 100) / 200}
                onChange={(value) =>
                  onSet("balance", Math.round(value * 200) - 100)
                }
              />
            </div>
          </div>

          <div className="hidden w-px shrink-0 bg-line sm:block" />

          <div className="flex min-w-0 flex-1 flex-col gap-[10px]">
            <SectionHeader title="REPLAYGAIN" />
            <div className="flex items-center gap-4">
              <div className="flex min-w-0 flex-1 flex-col justify-center gap-[14px]">
                <div className="flex items-center gap-[10px]">
                  <FieldLabel fixed>Mode</FieldLabel>
                  <Select
                    aria-label="ReplayGain mode"
                    options={REPLAYGAIN_MODES}
                    value={String(settings.replaygainType)}
                    className="w-[150px]"
                    onChange={(event) =>
                      onSet("rg_type", Number(event.target.value))
                    }
                  />
                </div>
                <div className="flex items-center gap-[10px]">
                  <FieldLabel>Prevent clipping</FieldLabel>
                  <Toggle
                    checked={settings.replaygainNoclip}
                    label="Prevent clipping"
                    onChange={(next) => onSet("rg_noclip", next ? 1 : 0)}
                  />
                </div>
              </div>
              <Knob
                label="Pre-amp"
                valueText={db(settings.replaygainPreamp)}
                norm={(settings.replaygainPreamp + 120) / 240}
                onChange={(value) =>
                  onSet("rg_preamp", Math.round(value * 48) * 5 - 120)
                }
              />
            </div>
          </div>
        </div>

        {/* ── Crossfade ─────────────────────────────────────────────────── */}
        <SectionHeader title="CROSSFADE" />
        <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
          <FieldLabel fixed>Mode</FieldLabel>
          <Select
            aria-label="Crossfade mode"
            options={CROSSFADE_MODES}
            value={String(settings.crossfade)}
            className="w-[230px]"
            onChange={(event) => onSet("crossfade", Number(event.target.value))}
          />
          <div className="hidden flex-1 lg:block" />
          <FieldLabel>Fade out</FieldLabel>
          <Select
            aria-label="Fade out mode"
            options={MIXMODES}
            value={String(settings.fadeOutMixmode)}
            className="w-[140px]"
            onChange={(event) =>
              onSet("fade_out_mixmode", Number(event.target.value))
            }
          />
          <FieldLabel>Dithering</FieldLabel>
          <Toggle
            checked={settings.dithering}
            label="Dithering"
            onChange={(next) => onSet("dithering", next ? 1 : 0)}
          />
        </div>

        <div
          className={`flex flex-wrap justify-center gap-[26px] pt-[2px] transition-opacity ${
            settings.crossfade === 0 ? "opacity-[0.35]" : ""
          }`}
        >
          <Knob
            label="In delay"
            valueText={`${settings.fadeInDelay} s`}
            norm={settings.fadeInDelay / 7}
            defaultNorm={0}
            onChange={(value) => onSet("fade_in_delay", Math.round(value * 7))}
          />
          <Knob
            label="In duration"
            valueText={`${settings.fadeInDuration} s`}
            norm={settings.fadeInDuration / 15}
            defaultNorm={2 / 15}
            onChange={(value) =>
              onSet("fade_in_duration", Math.round(value * 15))
            }
          />
          <Knob
            label="Out delay"
            valueText={`${settings.fadeOutDelay} s`}
            norm={settings.fadeOutDelay / 7}
            defaultNorm={0}
            onChange={(value) => onSet("fade_out_delay", Math.round(value * 7))}
          />
          <Knob
            label="Out duration"
            valueText={`${settings.fadeOutDuration} s`}
            norm={settings.fadeOutDuration / 15}
            defaultNorm={2 / 15}
            onChange={(value) =>
              onSet("fade_out_duration", Math.round(value * 15))
            }
          />
        </div>
      </div>
    )}
  </Dialog>
);

export default AudioSettings;
