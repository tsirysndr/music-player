import { useNavigate } from "react-router-dom";
import Tracks from "../../Components/Tracks";
import { useGetTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";

const TracksPage = () => {
  const { data, loading, error } = useGetTracksQuery();
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  return (
    <>
      {!loading && data && (
        <Tracks
          tracks={data.tracks.slice(0, 100).map((track) => ({
            id: track.id,
            title: track.title,
            artist: track.artists.map((artist) => artist.name).join(", "),
            album: track.album.title,
            time: formatTime(track.duration! * 1000),
          }))}
          onClickLibraryItem={(item) => navigate(`/${item}`)}
        />
      )}
    </>
  );
};

export default TracksPage;
