import styled from "@emotion/styled";
import { Cell, Grid } from "baseui/layout-grid";
import { FC } from "react";
import Artist from "../../Icons/Artist";

const ArtistCover = styled.img`
  height: 220px;
  width: 220px;
  border-radius: 110px;
  cursor: pointer;
`;

const Wrapper = styled.div`
  margin-top: 34px;
`;

const NoArtistCover = styled.div`
  height: 220px;
  width: 220px;
  border-radius: 110px;
  cursor: pointer;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #f3f3f3b9;
`;

const ArtistName = styled.div`
  font-size: 14px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
  cursor: pointer;
  margin-top: 20px;
  margin-bottom: 18px;
  text-align: center;
  width: 220px;
`;

export type ArtistsProps = {
  artists: any[];
  onClickArtist: (artist: any) => void;
};

const Artists: FC<ArtistsProps> = (props) => {
  const { artists, onClickArtist } = props;
  return (
    <Wrapper>
      <Grid gridColumns={[2, 3, 4]} gridMargins={[8, 16, 18]}>
        {artists.map((item) => (
          <Cell key={item.id}>
            {item.cover && (
              <ArtistCover
                src={item.cover}
                onClick={() => onClickArtist(item)}
              />
            )}
            {!item.cover && (
              <NoArtistCover onClick={() => onClickArtist(item)}>
                <Artist width={75} height={75} color="#a4a3a3" />
              </NoArtistCover>
            )}
            <ArtistName>{item.name}</ArtistName>
          </Cell>
        ))}
      </Grid>
    </Wrapper>
  );
};

export default Artists;
