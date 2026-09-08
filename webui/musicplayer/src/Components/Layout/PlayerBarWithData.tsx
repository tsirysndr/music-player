import { useAtom } from "jotai";
import { useEffect, useState } from "react";
import { fetcher } from "../../Api/fetcher";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { useDevices } from "../../Hooks/useDevices";
import { useVolume } from "../../Hooks/useVolume";
import {
  audioSettingsOpenAtom,
  fullPlayerOpenAtom,
  queueOpenAtom,
} from "../../State";
import PlayerBar from "./PlayerBar";

/**
 * Whether the station playing right now is bookmarked, and a toggle for it.
 *
 * The station may have been queued in an earlier session, so the state comes
 * from the saved list rather than from whatever queued it.
 */
const useRadioBookmark = (stationId?: string) => {
  const [bookmarked, setBookmarked] = useState(false);

  useEffect(() => {
    if (!stationId) {
      setBookmarked(false);
      return;
    }
    let active = true;
    fetcher<any, any>(`query { savedRadios { id } }`, {})()
      .then((data) => {
        if (active) {
          setBookmarked(
            (data.savedRadios || []).some(
              (station: { id: string }) => station.id === stationId
            )
          );
        }
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [stationId]);

  const toggle = async () => {
    // Optimistic: the heart should answer the click, not the round trip.
    setBookmarked((was) => !was);
    try {
      const data = await fetcher<any, any>(
        `mutation { toggleCurrentRadioBookmark }`,
        {}
      )();
      setBookmarked(!!data.toggleCurrentRadioBookmark);
    } catch {
      setBookmarked((was) => !was);
    }
  };

  return { bookmarked, toggle };
};

/** Connects the player bar to playback, likes, volume and the chrome flags. */
const PlayerBarWithData = ({
  onOpenDevices,
}: {
  onOpenDevices: () => void;
}) => {
  const { nowPlaying, play, pause, next, previous, seek } = usePlayback();
  const { isLiked, toggleLike } = useLikes();
  const [queueOpen, setQueueOpen] = useAtom(queueOpenAtom);
  const [audioOpen, setAudioOpen] = useAtom(audioSettingsOpenAtom);
  const [fullPlayerOpen, setFullPlayer] = useAtom(fullPlayerOpenAtom);
  const { volume, muted, change: changeVolume, toggleMute } = useVolume();
  const { currentCastDevice } = useDevices();

  const isRadio = !!nowPlaying?.id?.startsWith("radio:");
  const { bookmarked, toggle: toggleBookmark } = useRadioBookmark(
    isRadio ? nowPlaying?.id?.replace(/^radio:/, "") : undefined
  );

  return (
    <PlayerBar
      title={nowPlaying?.title}
      artist={nowPlaying?.artist}
      album={nowPlaying?.album}
      cover={nowPlaying?.cover}
      stopped={!nowPlaying?.title}
      playing={!!nowPlaying?.isPlaying}
      progress={nowPlaying?.progress ?? 0}
      duration={nowPlaying?.duration ?? 0}
      isRadio={isRadio}
      // For a station the heart is the bookmark, not a like.
      liked={isRadio ? bookmarked : !!nowPlaying?.id && isLiked(nowPlaying.id)}
      volume={volume}
      muted={muted}
      queueOpen={queueOpen}
      audioSettingsOpen={audioOpen}
      overlay={fullPlayerOpen}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onSeek={(positionMs) => seek(positionMs)}
      onToggleLike={() =>
        isRadio ? toggleBookmark() : nowPlaying?.id && toggleLike(nowPlaying.id)
      }
      onVolume={changeVolume}
      onToggleMute={toggleMute}
      onToggleQueue={() => setQueueOpen((open) => !open)}
      onOpenFullPlayer={() => setFullPlayer(true)}
      onOpenAudioSettings={() => setAudioOpen((open) => !open)}
      onOpenDevices={onOpenDevices}
      castingTo={currentCastDevice?.name}
    />
  );
};

export default PlayerBarWithData;
