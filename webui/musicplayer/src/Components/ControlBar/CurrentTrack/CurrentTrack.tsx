import styled from "@emotion/styled";
import { FC } from "react";
import { ProgressBar } from "baseui/progress-bar";

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

const CurrentTrack: FC = (props) => {
  return (
    <Container>
      <AlbumCover src="https://resources.tidal.com/images/543575fc/ad02/419b/ae61/671558dc019d/320x320.jpg" />
      <Wrapper>
        <TrackInfo>
          <Title>Otherside</Title>
          <Artist>Red Hot Chili Peppers - Californication</Artist>
        </TrackInfo>
        <ProgressBar
          value={30}
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
  );
};

export default CurrentTrack;
