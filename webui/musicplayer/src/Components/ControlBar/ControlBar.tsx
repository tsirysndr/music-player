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
import { useTheme } from "@emotion/react";
import DeviceList from "./DeviceList";
import { Device } from "../../Types/Device";

const Container = styled.div`
  display: flex;
  align-items: center;
  height: 96px;
  padding-left: 26px;
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
  flex: 1;
  justify-content: flex-end;
  padding-right: 25px;
`;

export type ControlBarProps = {
  nowPlaying?: {
    album: string;
    artist: string;
    title: string;
    cover: string;
    duration: number;
    progress: number;
    isPlaying?: boolean;
    albumId: string;
  };
  castDevices: Device[];
  currentCastDevice?: Device;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
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

  useEffect(() => {
    if (!!nowPlaying) {
      setPlayed(nowPlaying.isPlaying!);
    }
  }, [nowPlaying?.isPlaying]);

  useEffect(() => {
    setPlayQueueButtonColor(theme.colors.icon);
  }, [theme]);

  const handlePlay = () => {
    setPlayed(true);
    onPlay();
  };

  const handlePause = () => {
    setPlayed(false);
    onPause();
  };

  return (
    <Container>
      <Controls>
        <Button onClick={onShuffle}>
          <Shuffle color={theme.colors.text} />
        </Button>
        <Button onClick={onPrevious}>
          <Previous color={theme.colors.text} />
        </Button>
        {!played && (
          <Button onClick={handlePlay}>
            <Play color={theme.colors.text} />
          </Button>
        )}
        {played && (
          <Button onClick={handlePause}>
            <Pause color={theme.colors.text} />
          </Button>
        )}
        <Button onClick={onNext}>
          <Next color={theme.colors.text} />
        </Button>
        <Button onClick={onRepeat}>
          <Repeat color={theme.colors.text} />
        </Button>
      </Controls>
      <CurrentTrack nowPlaying={nowPlaying} />

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
            <Speaker size={18} color={theme.colors.icon} />
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
};

export default ControlBar;
