import { FC } from "react";
import Albums from "./Albums";
import { useGetAlbumsQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { resourceUriResolver } from "../../ResourceUriResolver";
import { useNavigate } from "react-router-dom";

const AlbumsWithData: FC = () => {
  const { data, loading } = useGetAlbumsQuery({
    fetchPolicy: "cache-and-network",
  });
  const navigate = useNavigate();
  const { currentCastDevice } = useDevices();
  const albums = !loading && data ? data.albums : [];
  return (
    <Albums
      albums={albums.map((album) => ({
        id: album.id,
        title: album.title,
        artist: album.artist,
        cover:
          album.cover && resourceUriResolver.resolve(`/covers/${album.cover}`),
      }))}
      onClickAlbum={({ id }) => navigate(`/albums/${id}`)}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default AlbumsWithData;
