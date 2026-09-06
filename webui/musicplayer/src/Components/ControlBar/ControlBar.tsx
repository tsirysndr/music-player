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

const Container = styled.div<{full?:boolean}>`
  display: flex;
  align-items: center;
  height: 96px;
  padding-left: 26px;
  position: relative;
  z-index: 11;
  background: ${p=>p.full?'rgba(12, 9, 18, .68)':'transparent'};
  backdrop-filter: ${p=>p.full?'blur(24px) saturate(1.25)':'none'};
  -webkit-backdrop-filter: ${p=>p.full?'blur(24px) saturate(1.25)':'none'};
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
const FullPlayer = styled.div<{cover?:string}>`
  position:fixed;inset:0 0 96px 0;z-index:10;background:#09070d;
  display:flex;align-items:center;justify-content:center;overflow:hidden;
  &:before{content:"";position:absolute;inset:-40px;background:${p=>p.cover?`url("${p.cover}") center/cover`:'none'};filter:blur(30px);opacity:.48;transform:scale(1.08)}
  &:after{content:"";position:absolute;inset:0;background:#07050bb8}
`;
const FullContent=styled.div`position:relative;z-index:1;text-align:center;color:white;`;
const FullCover=styled.img`width:min(58vh,520px);height:min(58vh,520px);object-fit:cover;box-shadow:0 22px 70px #000b;`;
const Back=styled(Button)`position:absolute;z-index:2;left:24px;top:24px;color:white;font-size:28px;`;

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

  useEffect(() => {
    if (!!nowPlaying) {
      setPlayed(nowPlaying.isPlaying!);
    }
  }, [nowPlaying?.isPlaying]);

  useEffect(() => {
    setPlayQueueButtonColor(theme.colors.icon);
  }, [theme]);

  useEffect(() => {
    document.body.classList.toggle("miniplayer-fullscreen", full);
    return () => document.body.classList.remove("miniplayer-fullscreen");
  }, [full]);

  const handlePlay = () => {
    setPlayed(true);
    onPlay();
  };

  const handlePause = () => {
    setPlayed(false);
    onPause();
  };

  return (<>
    {full && nowPlaying?.title && <FullPlayer cover={nowPlaying.cover}><Back onClick={()=>setFull(false)}>‹</Back><FullContent>{nowPlaying.cover&&<FullCover src={nowPlaying.cover}/>}</FullContent></FullPlayer>}
    <Container full={full}>
      <Controls>
        {!isRadio && <Button onClick={onShuffle}>
          <Shuffle color={theme.colors.text} />
        </Button>}
        {!isRadio && <Button onClick={onPrevious}>
          <Previous color={theme.colors.text} />
        </Button>}
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
        {!isRadio && <Button onClick={onNext}>
          <Next color={theme.colors.text} />
        </Button>}
        {!isRadio && <Button onClick={onRepeat}>
          <Repeat color={theme.colors.text} />
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
    </Container></>
  );
};

export default ControlBar;
