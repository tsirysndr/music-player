import { useNavigate } from "react-router-dom";
import Artists from "../../Components/Artists";

const ArtistsPage = () => {
  const navigate = useNavigate();
  return (
    <Artists
      artists={[]}
      onClickArtist={() => {}}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
    />
  );
};

export default ArtistsPage;
