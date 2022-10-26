import { useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import ArtistDetails from "../../Components/ArtistDetails";
import { useGetArtistLazyQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";

const ArtistDetailsPage = () => {
  const params = useParams();
  const [getArtist, { data, loading }] = useGetArtistLazyQuery({
    variables: {
      id: params.id!,
    },
    fetchPolicy: "network-only",
  });

  useEffect(() => {
    params.id && getArtist();
  }, [params.id, getArtist]);

  const { formatTime } = useTimeFormat();
  const navigate = useNavigate();
  const { play, pause, next, previous, nowPlaying } = usePlayback();
  const artist = !loading && data ? data.artist : {};
  const tracks =
    !loading && data
      ? data.artist.songs.map((track) => ({
          id: track.id,
          title: track.title,
          artist: track.artists.map((artist) => artist.name).join(", "),
          time: formatTime(track.duration! * 1000),
        }))
      : [];
  return (
    <ArtistDetails
      onBack={() => navigate(-1)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      artist={artist}
      tracks={tracks}
      albums={[]}
      nowPlaying={nowPlaying}
    />
  );
};

export default ArtistDetailsPage;
