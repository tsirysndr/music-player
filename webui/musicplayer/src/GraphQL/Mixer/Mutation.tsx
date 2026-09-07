import gql from "graphql-tag";
import { AUDIO_SETTINGS_FIELDS } from "./Query";

/**
 * Change one audio setting, by the names the daemon takes — `eq_enabled`,
 * `eq_precut`, `bass`, `treble`, `balance`, `rg_type`, `rg_preamp`,
 * `rg_noclip`, `crossfade`, `fade_in_delay`, `fade_in_duration`,
 * `fade_out_delay`, `fade_out_duration`, `fade_out_mixmode`, `dithering`.
 *
 * The whole state comes back, so the dialog renders what was stored rather
 * than what it asked for — the two differ wherever a value was clamped.
 */
export const SET_AUDIO_SETTING = gql`
  mutation SetAudioSetting($name: String!, $value: Int!) {
    setAudioSetting(name: $name, value: $value) {
      ...AudioSettingsFields
    }
  }
  ${AUDIO_SETTINGS_FIELDS}
`;

/** One EQ band's gain, in dB × 10 (-240..=240). */
export const SET_EQ_BAND_GAIN = gql`
  mutation SetEqBandGain($band: Int!, $gain: Int!) {
    setEqBandGain(band: $band, gain: $gain) {
      ...AudioSettingsFields
    }
  }
  ${AUDIO_SETTINGS_FIELDS}
`;
