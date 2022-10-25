import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import ArtistDetails from "../../Components/ArtistDetails";
import { usePlayback } from "../../Hooks/usePlayback";

const ArtistDetailsPage = () => {
  const navigate = useNavigate();
  const { play, pause, next, previous, nowPlaying } = usePlayback();
  return (
    <ArtistDetails
      onBack={() => {}}
      onClickLibraryItem={() => {}}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      artist={{}}
      tracks={[]}
      albums={[]}
      nowPlaying={nowPlaying}
    />
  );
};

export default ArtistDetailsPage;
