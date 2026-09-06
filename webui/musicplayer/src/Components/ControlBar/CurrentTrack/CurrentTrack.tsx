import styled from "@emotion/styled";
import { FC, useRef, useState } from "react";
import Track from "../../Icons/Track";
import { useCover } from "../../../Hooks/useCover";
import { useTimeFormat } from "../../../Hooks/useFormat";
import { Link } from "react-router-dom";
import RadioArt from "../../RadioArt";

const FlexContainer = styled.div`
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
`;

const Container = styled.div`
  height: 76px;
  min-width: 531px;
  max-width: 800px;
  display: flex;
  align-items: center;
  border: 1px solid ${(props) => props.theme.colors.currentTrackBorder};
  border-radius: 5px;
  margin-left: 30px;
  padding-left: 8px;
  flex: 1;
`;

const AlbumCover = styled.img`
  height: 62px;
  width: 62px;
`;

const Art = styled.span`
  cursor: pointer;
  display: flex;
  flex-shrink: 0;
`;

const Elapsed = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 10px;
  font-family: RockfordSansRegular;
  color: ${(props) => props.theme.colors.secondaryText};
`;

const LiveBadge = styled.span`
  font-size: 9px;
  letter-spacing: 0.08em;
  color: #ab28fc;
`;

const NoCover = styled.div`
  height: 62px;
  width: 62px;
  background-color: ${(props) => props.theme.colors.cover};
  display: flex;
  align-items: center;
  justify-content: center;
`;

const TrackInfo = styled.div`
  display: flex;
  flex-direction: column;
  width: 100%;
  min-width: 0;
  overflow: hidden;
`;

const Wrapper = styled.div`
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  min-width: 0;
  width: auto;
  flex: 1;
  overflow: hidden;
`;

const Artist = styled.div`
  text-align: center;
  font-family: RockfordSansLight;
  font-size: 14px;
  color: ${(props) => props.theme.colors.secondaryText};
  white-space: nowrap;
  text-overflow: ellipsis;
`;

const AlbumTitle = styled.span`
  color: ${(props) => props.theme.colors.secondaryText} !important;
  white-space: nowrap;
  text-overflow: ellipsis;
`;

const Title = styled.div`
  text-align: center;
  font-size: 14px;
  white-space: nowrap;
  text-overflow: ellipsis;
  color: ${(props) => props.theme.colors.text};
`;

const Row = styled.div`
  display: flex;
  flex-direction: row;
  width: 100%;
  align-items: center;
`;

const Time = styled.div`
  font-size: 10px;
  color: rgba(0, 0, 0, 0.542);
  font-family: RockfordSansRegular;
  text-align: center;
  width: 60px;
  margin-top: -3px;
  color: ${(props) => props.theme.colors.text};
`;

const ProgressbarContainer = styled.div`
  width: 88%;
`;

const SeekBarWrapper = styled.div`
  position: relative;
  width: 100%;
  height: 16px;
  display: flex;
  align-items: center;
  cursor: pointer;
  touch-action: none;
`;

const SeekBarTrack = styled.div`
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background-color: rgba(177, 178, 181, 0.218);
  overflow: hidden;
`;

const SeekBarProgress = styled.div`
  height: 100%;
  border-radius: 2px;
  background-color: #ab28fc;
`;

export type SeekBarProps = {
  progress: number;
  duration: number;
  onSeek?: (positionMs: number) => void;
};

const SeekBar: FC<SeekBarProps> = ({ progress, duration, onSeek }) => {
  const barRef = useRef<HTMLDivElement>(null);
  const [dragRatio, setDragRatio] = useState<number | null>(null);

  const ratioFromEvent = (e: React.PointerEvent): number => {
    const rect = barRef.current!.getBoundingClientRect();
    const ratio = (e.clientX - rect.left) / rect.width;
    return Math.min(Math.max(ratio, 0), 1);
  };

  const handlePointerDown = (e: React.PointerEvent) => {
    if (!onSeek || duration <= 0) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    setDragRatio(ratioFromEvent(e));
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    if (dragRatio === null) return;
    setDragRatio(ratioFromEvent(e));
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    if (dragRatio === null || !onSeek || duration <= 0) return;
    setDragRatio(null);
    onSeek(Math.round(ratioFromEvent(e) * duration));
  };

  const ratio =
    dragRatio !== null ? dragRatio : duration > 0 ? progress / duration : 0;

  return (
    <SeekBarWrapper
      ref={barRef}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
    >
      <SeekBarTrack>
        <SeekBarProgress
          style={{ width: `${Math.min(ratio * 100, 100)}%` }}
        />
      </SeekBarTrack>
    </SeekBarWrapper>
  );
};

const Placeholder = styled.div`
  color: #767676;
`;

const Separator = styled.span`
  margin-left: 8px;
  margin-right: 8px;
`;

export type CurrentTrackProps = {
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
  onSeek?: (positionMs: number) => void;
  onExpand?: () => void;
  hideArt?: boolean;
};

const CurrentTrack: FC<CurrentTrackProps> = ({
  nowPlaying,
  onSeek,
  onExpand,
  hideArt,
}) => {
  const { cover } = useCover(nowPlaying?.cover);
  const { formatTime } = useTimeFormat();
  const isRadio = !!nowPlaying?.id?.startsWith("radio:");
  const showStation =
    !!nowPlaying?.album &&
    nowPlaying.album !== nowPlaying.artist &&
    nowPlaying.album !== nowPlaying.title;
  return (
    <FlexContainer>
      {(!nowPlaying || !nowPlaying!.title) && (
        <Container>
          <NoCover>
            <Track width={28} height={28} color="#a4a3a3" />
          </NoCover>
          <Wrapper>
            <Placeholder>No song is currently playing</Placeholder>
          </Wrapper>
        </Container>
      )}
      {nowPlaying && nowPlaying!.title && (
        <Container>
          {!hideArt && (isRadio || cover) && (
            <Art onClick={onExpand}>
              {isRadio ? (
                <RadioArt logo={nowPlaying.cover} size={62} radius={0} />
              ) : (
                <AlbumCover src={cover} />
              )}
            </Art>
          )}
          <Wrapper>
            <TrackInfo>
              <Title>{nowPlaying?.title}</Title>
              <Artist>
                <span>{nowPlaying?.artist}</span>
                {!isRadio && (
                  <>
                    <Separator>-</Separator>
                    <Link to={`/albums/${nowPlaying!.albumId}`}>
                      <AlbumTitle>{nowPlaying?.album}</AlbumTitle>
                    </Link>
                  </>
                )}
                {/* For a station the album slot carries the station name; it
                    is only worth a line once ICY metadata has replaced the
                    title with the song actually on the air (before that the
                    title already IS the station name). */}
                {isRadio && showStation && (
                  <>
                    <Separator>·</Separator>
                    <AlbumTitle>{nowPlaying?.album}</AlbumTitle>
                  </>
                )}
              </Artist>
            </TrackInfo>
            {!isRadio && (
              <Row>
                <Time>{formatTime(nowPlaying?.progress)}</Time>
                <ProgressbarContainer>
                  <SeekBar
                    progress={nowPlaying!.progress}
                    duration={nowPlaying!.duration}
                    onSeek={onSeek}
                  />
                </ProgressbarContainer>
                <Time>{formatTime(nowPlaying?.duration)}</Time>
              </Row>
            )}
            {isRadio && (
              <Elapsed>
                <LiveBadge>LIVE</LiveBadge>
                <span>{formatTime(nowPlaying!.progress)}</span>
              </Elapsed>
            )}
          </Wrapper>
        </Container>
      )}
    </FlexContainer>
  );
};

export default CurrentTrack;
