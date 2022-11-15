import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC } from "react";
import { useCover } from "../../../Hooks/useCover";
import AlbumIcon from "../../Icons/AlbumCover";

const AlbumCover = styled.img`
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

const Wrapper = styled.div`
  margin-top: 34px;
`;

export type AlbumProps = {
  onClick: (item: any) => void;
  album: any;
};

const Album: FC<AlbumProps> = ({ onClick, album }) => {
  const { cover } = useCover(album.cover);
  return (
    <Wrapper>
      {cover && <AlbumCover src={cover} onClick={() => onClick(album)} />}
      {!cover && (
        <NoAlbumCover onClick={() => onClick(album)}>
          <AlbumIcon />
        </NoAlbumCover>
      )}
      <Title onClick={() => onClick(album)}>{album.title}</Title>
      <Artist>{album.artist}</Artist>
    </Wrapper>
  );
};

export type AlbumsProps = {
  albums: any[];
  onClickAlbum: (album: any) => void;
};

const Albums: FC<AlbumsProps> = (props) => {
  const { albums, onClickAlbum } = props;
  return (
    <>
      <Grid gridColumns={[2, 3, 4, 6]} gridMargins={[8, 16, 18]}>
        {albums.map((item) => (
          <Cell key={item.id}>
            <Album onClick={onClickAlbum} album={item} />
          </Cell>
        ))}
      </Grid>
    </>
  );
};

export default Albums;
