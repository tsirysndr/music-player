import styled from "@emotion/styled";
import { FC, useEffect, useState } from "react";
import Next from "../Icons/Next";
import Pause from "../Icons/Pause";
import Play from "../Icons/Play";
import Previous from "../Icons/Previous";
import Repeat from "../Icons/Repeat";
import Shuffle from "../Icons/Shuffle";
import CurrentTrack from "./CurrentTrack";
import { ListOutline } from "@styled-icons/evaicons-outline";
import { Speaker } from "@styled-icons/bootstrap";
import { StatefulPopover } from "baseui/popover";
import PlayQueue from "./PlayQueue";
import { Track } from "../../Types";
import { ThemeProvider, useTheme } from "@emotion/react";
import DeviceList from "./DeviceList";
import { Device } from "../../Types/Device";
import RadioArt from "../RadioArt";
import Heart from "../Icons/Heart";
import HeartOutline from "../Icons/HeartOutline";
import { fetcher } from "../../Api/fetcher";
import { FullscreenOverlayTheme } from "../../Theme";

const Container = styled.div<{ full?: boolean }>`
  display: flex;
  align-items: center;
  height: 96px;
  padding-left: 26px;
  position: relative;
  z-index: 11;
  background: transparent;

  ${(props) =>
    props.full &&
    `
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(12, 9, 18, 0.68);
    backdrop-filter: blur(24px) saturate(1.25);
    -webkit-backdrop-filter: blur(24px) saturate(1.25);
  `}
`;

const Controls = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 32px;
  width: 200px;
`;

const Button = styled.button`
  background-color: transparent;
  cursor: pointer;
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  &:hover {
    opacity: 0.6;
  }
`;

const PlayQueueButton = styled(Button)``;

const SpeakerButton = styled(Button)`
  margin-right: 10px;
`;

const ButtonGroup = styled.div`
  display: flex;
  justify-content: flex-end;
  padding-right: 25px;
  width: 200px;
`;

const FullPlayer = styled.div<{ cover?: string }>`
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 10;
  background: #09070d;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding-bottom: 96px;
  box-sizing: border-box;

  &:before {
    content: "";
    position: absolute;
    top: -40px;
    left: -40px;
    right: -40px;
    bottom: -40px;
    background: ${(props) =>
      props.cover ? `url("${props.cover}") center / cover` : "none"};
    filter: blur(30px);
    opacity: 0.48;
    transform: scale(1.08);
  }

  &:after {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(7, 5, 11, 0.72);
  }
`;

const FullContent = styled.div`
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  color: #fff;
  max-width: min(90vw, 620px);
`;

const FullCover = styled.img`
  width: min(58vh, 520px);
  height: min(58vh, 520px);
  object-fit: cover;
  border-radius: 6px;
  box-shadow: 0 22px 70px rgba(0, 0, 0, 0.73);
`;

const FullTitle = styled.div`
  margin-top: 28px;
  font-family: RockfordSansBold;
  font-size: 24px;
  color: #fff;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const FullArtist = styled.div`
  margin-top: 8px;
  font-family: RockfordSansLight;
  font-size: 15px;
  color: rgba(255, 255, 255, 0.66);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const Back = styled(Button)`
  position: absolute;
  z-index: 12;
  left: 24px;
  top: 24px;
  color: #fff;
  font-size: 28px;
  line-height: 1;
`;

/// Whether the station playing right now is bookmarked, and a toggle for it.
/// The station may have been queued in an earlier session, so the state comes
/// from the saved list rather than from whatever queued it.
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
            (data.savedRadios || []).some((s: { id: string }) => s.id === stationId)
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

export type ControlBarProps = {
  nowPlaying?: {
    album?: string;
    artist?: string;
    title?: string;
    cover?: string;
    duration: number;
    progress: number;
    isPlaying?: boolean;
    albumId?: string;
    id?: string;
  };
  castDevices: Device[];
  currentCastDevice?: Device;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  onSeek?: (positionMs: number) => void;
  nextTracks?: Track[];
  previousTracks?: Track[];
  onPlayTrackAt: (position: number) => void;
  onRemoveTrackAt: (position: number) => void;
  connectToCastDevice: (deviceId: string) => void;
  disconnectFromCastDevice: () => void;
};

const ControlBar: FC<ControlBarProps> = (props) => {
  const theme = useTheme();
  const [played, setPlayed] = useState(false);
  const [full, setFull] = useState(false);
  const [playQueueButtonColor, setPlayQueueButtonColor] = useState(
    theme.colors.icon
  );
  const {
    nowPlaying,
    onNext,
    onPrevious,
    onPlay,
    onPause,
    onShuffle,
    onRepeat,
  } = props;
  const isRadio = !!nowPlaying?.id?.startsWith("radio:");
  // While the fullscreen player is open the bar floats over a dark backdrop,
  // so it has to stop reading colors from the (possibly light) active theme.
  const barTheme = full ? FullscreenOverlayTheme : theme;
  const { bookmarked, toggle: toggleBookmark } = useRadioBookmark(
    isRadio ? nowPlaying?.id?.replace(/^radio:/, "") : undefined
  );

  useEffect(() => {
    if (!!nowPlaying) {
      setPlayed(nowPlaying.isPlaying!);
    }
  }, [nowPlaying?.isPlaying]);

  useEffect(() => {
    setPlayQueueButtonColor(barTheme.colors.icon);
  }, [barTheme]);

  useEffect(() => {
    document.body.classList.toggle("miniplayer-fullscreen", full);
    return () => document.body.classList.remove("miniplayer-fullscreen");
  }, [full]);

  // Nothing left to show fullscreen once playback is cleared: collapse rather
  // than leaving the bar pinned over an empty page.
  useEffect(() => {
    if (full && !nowPlaying?.title) {
      setFull(false);
    }
  }, [full, nowPlaying?.title]);

  const handlePlay = () => {
    setPlayed(true);
    onPlay();
  };

  const handlePause = () => {
    setPlayed(false);
    onPause();
  };

  const bar = (
    <Container full={full}>
      <Controls>
        {!isRadio && <Button onClick={onShuffle}>
          <Shuffle color={barTheme.colors.text} />
        </Button>}
        {!isRadio && <Button onClick={onPrevious}>
          <Previous color={barTheme.colors.text} />
        </Button>}
        {!played && (
          <Button onClick={handlePlay}>
            <Play color={barTheme.colors.text} />
          </Button>
        )}
        {played && (
          <Button onClick={handlePause}>
            <Pause color={barTheme.colors.text} />
          </Button>
        )}
        {isRadio && (
          <Button onClick={toggleBookmark} title="Bookmark station">
            {bookmarked ? (
              <Heart size={22} color="#fe099c" />
            ) : (
              <HeartOutline size={22} color={barTheme.colors.icon} />
            )}
          </Button>
        )}
        {!isRadio && <Button onClick={onNext}>
          <Next color={barTheme.colors.text} />
        </Button>}
        {!isRadio && <Button onClick={onRepeat}>
          <Repeat color={barTheme.colors.text} />
        </Button>}
      </Controls>
      <CurrentTrack nowPlaying={nowPlaying} onSeek={props.onSeek} onExpand={()=>setFull(true)} hideArt={full} />

      <ButtonGroup>
        <StatefulPopover
          placement="bottom"
          content={({ close }) => <DeviceList {...props} close={close} />}
          overrides={{
            Body: {
              style: {
                left: "-70px",
              },
            },
            Inner: {
              style: {
                backgroundColor: theme.colors.popoverBackground,
              },
            },
          }}
        >
          <SpeakerButton>
            <Speaker size={18} color={barTheme.colors.icon} />
          </SpeakerButton>
        </StatefulPopover>
        <StatefulPopover
          onOpen={() => setPlayQueueButtonColor("#ab28fc")}
          onClose={() => setPlayQueueButtonColor(theme.colors.icon)}
          placement="bottom"
          content={() => <PlayQueue {...props} />}
          overrides={{
            Body: {
              style: {
                left: "-21px",
              },
            },
            Inner: {
              style: {
                backgroundColor: theme.colors.popoverBackground,
              },
            },
          }}
        >
          <PlayQueueButton>
            <ListOutline size={24} color={playQueueButtonColor} />
          </PlayQueueButton>
        </StatefulPopover>
      </ButtonGroup>
    </Container>
  );

  return (
    <ThemeProvider theme={barTheme}>
      {full && nowPlaying?.title && (
        <FullPlayer cover={nowPlaying.cover}>
          <Back onClick={() => setFull(false)}>‹</Back>
          <FullContent>
            {isRadio ? (
              <RadioArt logo={nowPlaying.cover} size={340} radius={6} />
            ) : (
              nowPlaying.cover && <FullCover src={nowPlaying.cover} alt="" />
            )}
            <FullTitle>{nowPlaying.title}</FullTitle>
            <FullArtist>
              {/* A station keeps its name in the album slot, so it still
                  reads "Artist - Station" once ICY metadata names the song.
                  Before any metadata arrives the title already IS the station
                  name, so drop the repeat. */}
              {[nowPlaying.artist, nowPlaying.album]
                .filter(
                  (part, i, all) =>
                    !!part &&
                    all.indexOf(part) === i &&
                    part !== nowPlaying.title
                )
                .join(" - ")}
            </FullArtist>
          </FullContent>
        </FullPlayer>
      )}
      {bar}
    </ThemeProvider>
  );
};

export default ControlBar;
