import { useNavigate } from "react-router-dom";
import Artists from "../../Components/Artists";
import { useGetArtistsQuery } from "../../Hooks/GraphQL";
import { usePlayback } from "../../Hooks/usePlayback";

const ArtistsPage = () => {
  const { data, loading } = useGetArtistsQuery();
  const navigate = useNavigate();
  const { play, pause, next, previous, nowPlaying } = usePlayback();
  const artists = !loading && data ? data.artists : [];
  return (
    <Artists
      artists={artists.map((artist) => ({
        id: artist.id,
        name: artist.name,
        cover: artist.picture,
      }))}
      onClickArtist={({ id }) => navigate(`/artists/${id}`)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      nowPlaying={nowPlaying}
    />
  );
};

export default ArtistsPage;
