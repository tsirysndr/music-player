import styled from "@emotion/styled";
import { FC, useMemo } from "react";
import { useCover } from "../../Hooks/useCover";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";
import AlbumIcon from "../Icons/AlbumCover";
import { Device } from "../../Types/Device";
import ListeningOn from "../ListeningOn";
import { FixedSizeGrid as Grid } from "react-window";
import { LazyLoadImage } from "react-lazy-load-image-component";

const Container = styled.div`
  display: flex;
  flex-direction: row;
  background-color: ${(props) => props.theme.colors.background};
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

const AlbumCover = styled(LazyLoadImage)`
  height: 220px;
  width: 220px;
  border-radius: 3px;
  cursor: pointer;
`;

const NoAlbumCover = styled.div`
  height: 220px;
  width: 220px;
  border-radius: 3px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #ddaefb14;
`;

const Wrapper = styled.div`
  margin-top: 34px;
  margin-left: 10px;
`;

const Artist = styled.div`
  color: #828282;
  margin-bottom: 56px;
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
`;

const Title = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  color: ${(props) => props.theme.colors.text};
`;

const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;
`;

export type AlbumsProps = {
  albums: any[];
  onClickAlbum: (album: any) => void;
  currentCastDevice?: Device;
  onFilter: (filter: string) => void;
};

export type AlbumProps = {
  onClick: (item: any) => void;
  album: any;
  currentCastDevice?: Device;
};

const Album: FC<AlbumProps> = ({ onClick, album }) => {
  const { cover } = useCover(album.cover);
  return (
    <>
      {cover && (
        <AlbumCover
          src={cover}
          onClick={() => onClick(album)}
          placeholder={
            <NoAlbumCover>
              <AlbumIcon />
            </NoAlbumCover>
          }
        />
      )}
      {!cover && (
        <NoAlbumCover onClick={() => onClick(album)}>
          <AlbumIcon />
        </NoAlbumCover>
      )}
      <Title onClick={() => onClick(album)}>{album.title}</Title>
      <Artist>{album.artist}</Artist>
    </>
  );
};

const Albums: FC<AlbumsProps> = (props) => {
  const { albums, onClickAlbum, currentCastDevice, onFilter } = props;
  // convert albums array to matrix of 4 columns using reduce
  const data = useMemo(
    () =>
      albums.reduce((resultArray, item, index) => {
        const chunkIndex = Math.floor(index / 4);
        if (!resultArray[chunkIndex]) {
          resultArray[chunkIndex] = []; // start a new chunk
        }
        resultArray[chunkIndex].push(item);
        return resultArray;
      }, []),
    [albums]
  );

  const Cell = ({ rowIndex, columnIndex, style }: any) => (
    <>
      {data[rowIndex][columnIndex] && (
        <div style={style}>
          <Album onClick={onClickAlbum} album={data[rowIndex][columnIndex]} />
        </div>
      )}
    </>
  );

  const vh = (percent: number) => {
    const h = Math.max(
      document.documentElement.clientHeight,
      window.innerHeight || 0
    );
    return (percent * h) / 100;
  };

  const vw = (percent: number) => {
    const w = Math.max(
      document.documentElement.clientWidth,
      window.innerWidth || 0
    );
    return (percent * w) / 100;
  };

  return (
    <>
      {currentCastDevice && <ListeningOn deviceName={currentCastDevice.name} />}
      <Container>
        <Sidebar active="albums" />
        <Content>
          <ControlBar />
          <Scrollable>
            <MainContent
              title="Albums"
              placeholder="Filter Albums"
              onFilter={onFilter}
            >
              <Wrapper>
                <Grid
                  columnCount={4}
                  columnWidth={247}
                  rowCount={data.length}
                  rowHeight={319}
                  height={Math.max(
                    Math.min(data.length, 3) * 319,
                    vh(100) - 100
                  )}
                  width={vw(100) - 300}
                >
                  {Cell}
                </Grid>
              </Wrapper>
            </MainContent>
          </Scrollable>
        </Content>
      </Container>
    </>
  );
};

Albums.defaultProps = {
  albums: [],
};

export default Albums;
