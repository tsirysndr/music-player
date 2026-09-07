import gql from "graphql-tag";

/**
 * The DSP chain's state, in the firmware's own units — dB × 10 wherever a
 * fraction is meaningful, so a slider never round-trips a float.
 *
 * The ranges come back with the values because they are what a slider needs
 * and they are not derivable from the field alone.
 */
export const AUDIO_SETTINGS_FIELDS = gql`
  fragment AudioSettingsFields on AudioSettingsState {
    eqEnabled
    eqPrecut
    eqBands {
      cutoff
      q
      gain
    }
    bass
    bassMin
    bassMax
    treble
    trebleMin
    trebleMax
    balance
    replaygainType
    replaygainPreamp
    replaygainNoclip
    crossfade
    fadeInDelay
    fadeInDuration
    fadeOutDelay
    fadeOutDuration
    fadeOutMixmode
    dithering
  }
`;

export const GET_AUDIO_SETTINGS = gql`
  query GetAudioSettings {
    audioSettings {
      ...AudioSettingsFields
    }
  }
  ${AUDIO_SETTINGS_FIELDS}
`;
