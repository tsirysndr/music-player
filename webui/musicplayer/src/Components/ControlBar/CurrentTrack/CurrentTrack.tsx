import styled from "@emotion/styled";
import { FC } from "react";
import { ProgressBar } from "baseui/progress-bar";
import Track from "../../Icons/Track";
import { useCover } from "../../../Hooks/useCover";
import { useTimeFormat } from "../../../Hooks/useFormat";

const Container = styled.div`
  height: 76px;
  width: 531px;
  display: flex;
  align-items: center;
  border: 1px solid rgba(177, 178, 181, 0.25);
  border-radius: 5px;
  margin-left: 30px;
  padding-left: 8px;
`;

const AlbumCover = styled.img`
  height: 62px;
  width: 62px;
`;

const NoCover = styled.div`
  height: 62px;
  width: 62px;
  background-color: #f3f3f3b9;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const TrackInfo = styled.div`
  display: flex;
  flex-direction: column;
  width: 90%;
  overflow: hidden;
`;

const Wrapper = styled.div`
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  width: 90%;
  overflow: hidden;
`;

const Artist = styled.div`
  text-align: center;
  font-family: RockfordSansLight;
  font-size: 14px;
  color: rgba(0, 0, 0, 0.542);
  white-space: nowrap;
  text-overflow: ellipsis;
`;

const Title = styled.div`
  text-align: center;
  font-size: 14px;
  white-space: nowrap;
  text-overflow: ellipsis;
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
`;

const ProgressbarContainer = styled.div`
  width: 88%;
`;

const Placeholder = styled.div`
  color: #767676;
`;

export type CurrentTrackProps = {
  nowPlaying?: {
    album: string;
    artist: string;
    title: string;
    cover: string;
    duration: number;
    progress: number;
  };
};

const CurrentTrack: FC<CurrentTrackProps> = ({ nowPlaying }) => {
  const { cover } = useCover(nowPlaying?.cover);
  const { formatTime } = useTimeFormat();
  return (
    <>
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
          {cover && <AlbumCover src={cover} />}
          {!cover && (
            <NoCover>
              <Track width={28} height={28} color="#a4a3a3" />
            </NoCover>
          )}
          <Wrapper>
            <TrackInfo>
              <Title>{nowPlaying?.title}</Title>
              <Artist>
                {nowPlaying?.artist} - {nowPlaying?.album}
              </Artist>
            </TrackInfo>
            <Row>
              <Time>{formatTime(nowPlaying?.progress)}</Time>
              <ProgressbarContainer>
                <ProgressBar
                  value={
                    nowPlaying!.duration > 0
                      ? (nowPlaying!.progress / nowPlaying!.duration) * 100
                      : 0
                  }
                  overrides={{
                    BarProgress: {
                      style: () => ({
                        backgroundColor: "#ab28fc",
                      }),
                    },
                    Bar: {
                      style: () => ({
                        backgroundColor: "rgba(177, 178, 181, 0.218)",
                      }),
                    },
                  }}
                />
              </ProgressbarContainer>
              <Time>{formatTime(nowPlaying?.duration)}</Time>
            </Row>
          </Wrapper>
        </Container>
      )}
    </>
  );
};

export default CurrentTrack;
