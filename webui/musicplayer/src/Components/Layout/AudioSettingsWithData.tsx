import { useQueryClient } from "@tanstack/react-query";
import { useAtom } from "jotai";
import {
  useGetAudioSettingsQuery,
  useSetAudioSettingMutation,
  useSetEqBandGainMutation,
} from "../../Hooks/GraphQL";
import { audioSettingsOpenAtom } from "../../State";
import AudioSettings from "./AudioSettings";

/**
 * Connects the audio-settings dialog to the daemon's mixer.
 *
 * Every mutation returns the whole state, so the response is written straight
 * into the query cache rather than triggering a refetch — dragging an EQ band
 * fires a write per pointer move, and a round trip each would lag the fader
 * behind the finger.
 */
const AudioSettingsWithData = () => {
  const [open, setOpen] = useAtom(audioSettingsOpenAtom);
  const queryClient = useQueryClient();

  const { data, isLoading, error } = useGetAudioSettingsQuery(undefined, {
    enabled: open,
  });

  const cache = (settings: unknown) =>
    queryClient.setQueryData(useGetAudioSettingsQuery.getKey(), {
      audioSettings: settings,
    });

  const setSetting = useSetAudioSettingMutation({
    onSuccess: (result) => cache(result.setAudioSetting),
  });
  const setEqBand = useSetEqBandGainMutation({
    onSuccess: (result) => cache(result.setEqBandGain),
  });

  return (
    <AudioSettings
      isOpen={open}
      settings={data?.audioSettings}
      loading={isLoading}
      error={error instanceof Error ? error.message : undefined}
      onClose={() => setOpen(false)}
      onSet={(name, value) => setSetting.mutate({ name, value })}
      onSetEqBand={(band, gain) => setEqBand.mutate({ band, gain })}
    />
  );
};

export default AudioSettingsWithData;
