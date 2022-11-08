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
import { StatefulPopover } from "baseui/popover";
import PlayQueue from "./PlayQueue";
import { Track } from "../../Types";

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
};

const ControlBar: FC<ControlBarProps> = (props) => {
  const [played, setPlayed] = useState(false);
  const [playQueueButtonColor, setPlayQueueButtonColor] = useState("#000");
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
    if (nowPlaying) {
      setPlayed(nowPlaying.isPlaying!);
    }
  }, [nowPlaying]);

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
          <Shuffle />
        </Button>
        <Button onClick={onPrevious}>
          <Previous />
        </Button>
        {!played && (
          <Button onClick={handlePlay}>
            <Play />
          </Button>
        )}
        {played && (
          <Button onClick={handlePause}>
            <Pause />
          </Button>
        )}
        <Button onClick={onNext}>
          <Next />
        </Button>
        <Button onClick={onRepeat}>
          <Repeat />
        </Button>
      </Controls>
      <CurrentTrack nowPlaying={nowPlaying} />
      <ButtonGroup>
        <StatefulPopover
          onOpen={() => setPlayQueueButtonColor("#ab28fc")}
          onClose={() => setPlayQueueButtonColor("#000")}
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
                backgroundColor: "#fff",
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
