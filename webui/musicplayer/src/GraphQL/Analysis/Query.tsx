import gql from "graphql-tag";

/**
 * How a track sounds.
 *
 * `analyzed` false means nothing has been computed for it yet — the waveform is
 * then empty and every measurement null, rather than zero. Clients must check
 * it rather than reading a flat waveform as a silent track.
 */
export const GET_TRACK_ANALYSIS = gql`
  query GetTrackAnalysis($trackId: String!) {
    trackAnalysis(trackId: $trackId) {
      trackId
      analyzed
      waveform
      bpm
      bpmConfidence
      valence
      arousal
      lufs
      duration
      moods {
        name
        confidence
      }
    }
  }
`;
