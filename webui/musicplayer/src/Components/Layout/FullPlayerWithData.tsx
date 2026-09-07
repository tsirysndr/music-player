import { useAtom } from "jotai";
import { useEffect } from "react";
import { usePlayback } from "../../Hooks/usePlayback";
import { fullPlayerOpenAtom } from "../../State";
import FullPlayer from "./FullPlayer";

/** Connects the full player to playback. */
const FullPlayerWithData = () => {
  const [open, setOpen] = useAtom(fullPlayerOpenAtom);
  const { nowPlaying } = usePlayback();

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
    />
  );
};

export default FullPlayerWithData;
