import { useAtom, useAtomValue } from "jotai";
import { useEffect } from "react";
import { useGetTrackAnalysisQuery } from "../../Hooks/GraphQL";
import { useLevels } from "../../Hooks/useLevels";
import { usePlayback } from "../../Hooks/usePlayback";
import { fullPlayerOpenAtom, playbackPositionAtom } from "../../State";
import FullPlayer from "./FullPlayer";

/** Connects the full player to playback, the track's analysis, and the meter. */
const FullPlayerWithData = () => {
  const [open, setOpen] = useAtom(fullPlayerOpenAtom);
  const { nowPlaying, seek } = usePlayback();
  const position = useAtomValue(playbackPositionAtom);
  // The meter settles to nothing while paused rather than freezing mid-bounce.
  const levels = useLevels(!!nowPlaying?.isPlaying);

  const trackId = nowPlaying?.id ?? "";
  const { data } = useGetTrackAnalysisQuery(
    { trackId },
    {
      // Only while the canvas is open, and only for a real track: a waveform
      // nobody can see is not worth a request, and radio has none to fetch.
      enabled: open && !!trackId && !trackId.startsWith("radio:"),
      // The analysis of a given track never changes — it is derived from the
      // audio — so there is nothing to refetch once it has been read.
      staleTime: Infinity,
    }
  );

  const analysis = data?.trackAnalysis;
  // Length as decoded, which is what the waveform was drawn against. The tag's
  // duration can disagree, and seeking by it would land in the wrong place.
  const duration = analysis?.analyzed
    ? analysis.duration
    : (nowPlaying?.duration ?? 0);

  // Nothing left to show once playback is cleared: collapse rather than
  // leaving an empty canvas pinned over the page.
  useEffect(() => {
    if (open && !nowPlaying?.title) setOpen(false);
  }, [open, nowPlaying?.title, setOpen]);

  return (
    <FullPlayer
      open={open}
      title={nowPlaying?.title}
      cover={nowPlaying?.cover}
      isRadio={!!nowPlaying?.id?.startsWith("radio:")}
      onClose={() => setOpen(false)}
      waveform={analysis?.analyzed ? analysis.waveform : []}
      progress={duration > 0 ? position / 1000 / duration : 0}
      duration={duration}
      onSeek={(seconds) => seek(Math.round(seconds * 1000))}
      levels={levels}
      isPlaying={!!nowPlaying?.isPlaying}
    />
  );
};

export default FullPlayerWithData;
