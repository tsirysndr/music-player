import styled from "@emotion/styled";
import { Cell, Grid, STYLE } from "baseui/layout-grid";
import { FC } from "react";
import ControlBar from "../ControlBar";
import MainContent from "../MainContent";
import Sidebar from "../Sidebar";

const Container = styled.div`
  display: flex;
  flex-direction: row;
`;

const Content = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
`;

const AlbumCover = styled.img`
  height: 220px;
  width: 220px;
  border-radius: 3px;
  cursor: pointer;
`;

const Wrapper = styled.div`
  margin-top: 34px;
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
`;

const Scrollable = styled.div`
  height: calc(100vh - 100px);
  overflow-y: auto;
`;

export type AlbumProps = {
  albums: any[];
  onClickAlbum: (album: any) => void;
  onClickLibraryItem: (item: string) => void;
  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  nowPlaying: any;
};

const Albums: FC<AlbumProps> = (props) => {
  const { albums, onClickAlbum, onClickLibraryItem } = props;
  return (
    <Container>
      <Sidebar active="albums" onClickLibraryItem={onClickLibraryItem} />
      <Content>
        <ControlBar {...props} />
        <Scrollable>
          <MainContent title="Albums" placeholder="Filter Albums">
            <Wrapper>
              <Grid gridColumns={[2, 3, 4, 6]} gridMargins={[8, 16, 18]}>
                {albums.map((item) => (
                  <Cell key={item.id}>
                    <AlbumCover
                      src={item.cover}
                      onClick={() => onClickAlbum(item)}
                    />
                    <Title onClick={() => onClickAlbum(item)}>
                      {item.title}
                    </Title>
                    <Artist>{item.artist}</Artist>
                  </Cell>
                ))}
              </Grid>
            </Wrapper>
          </MainContent>
        </Scrollable>
      </Content>
    </Container>
  );
};

Albums.defaultProps = {
  albums: [],
};

export default Albums;
