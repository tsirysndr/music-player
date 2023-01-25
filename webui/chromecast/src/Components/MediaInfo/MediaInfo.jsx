import styled from "@emotion/styled";

const Container = styled.div`
  padding-left: 5vw;
  padding-right: 5vw;
  display: flex;
`;

const Cover = styled.img`
  height: 30vh;
  width: 30vh;
`;

const Artist = styled.div`
  font-size: 18px;
`;

const Title = styled.div`
  font-family: RockfordSansBold;
  font-size: 2.5em;
`;

const Metadata = styled.div`
  padding-left: 40px;
  display: flex;
  flex-direction: column;
  justify-content: end;
  height: 30vh;
`;

const MediaInfo = ({ artist, title, images }) => {
  return (
    <Container>
      <Cover src={images.length ? images[0].url : ""} />
      <Metadata>
        <div>
          <Title>{title}</Title>
          <Artist>{artist}</Artist>
        </div>
      </Metadata>
    </Container>
  );
};

MediaInfo.defaultProps = {
  artist: "",
  title: "",
  images: [],
};

export default MediaInfo;
