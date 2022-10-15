import styled from "@emotion/styled";
import { FC } from "react";
import { ProgressBar } from "baseui/progress-bar";
import Track from "../../Icons/Track";

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
`;

const Wrapper = styled.div`
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
`;

const Artist = styled.div`
  text-align: center;
  font-family: RockfordSansLight;
  color: rgba(0, 0, 0, 0.542);
`;

const Title = styled.div`
  text-align: center;
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
  return (
    <>
      {!nowPlaying && (
        <Container>
          <NoCover>
            <Track width={28} height={28} color="#a4a3a3" />
          </NoCover>
          <Wrapper>
            <Placeholder>No song is currently playing</Placeholder>
          </Wrapper>
        </Container>
      )}
      {nowPlaying && (
        <Container>
          <AlbumCover src={nowPlaying?.cover} />
          <Wrapper>
            <TrackInfo>
              <Title>{nowPlaying?.title}</Title>
              <Artist>
                {nowPlaying?.artist} - {nowPlaying?.album}
              </Artist>
            </TrackInfo>
            <ProgressBar
              value={
                nowPlaying!.duration > 0
                  ? (nowPlaying!.progress * 100) / nowPlaying!.duration
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
          </Wrapper>
        </Container>
      )}
    </>
  );
};

export default CurrentTrack;
