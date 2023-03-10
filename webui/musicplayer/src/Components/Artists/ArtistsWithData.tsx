import { FC } from "react";
import { useNavigate } from "react-router-dom";
import Artists from "./Artists";
import { useGetArtistsQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";

const ArtistsWithData: FC = () => {
  const { data, loading } = useGetArtistsQuery();
  const navigate = useNavigate();
  const { currentCastDevice } = useDevices();
  const artists = !loading && data ? data.artists : [];
  return (
    <Artists
      artists={artists.map((artist) => ({
        id: artist.id,
        name: artist.name,
        cover: artist.picture,
      }))}
      onClickArtist={({ id }) => navigate(`/artists/${id}`)}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default ArtistsWithData;
