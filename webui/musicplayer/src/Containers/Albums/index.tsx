import { useNavigate } from "react-router-dom";
import Albums from "../../Components/Albums";
import { useGetAlbumsQuery } from "../../Hooks/GraphQL";

const AlbumsPage = () => {
  const { data, loading, error } = useGetAlbumsQuery();
  const navigate = useNavigate();
  return (
    <Albums
      albums={[]}
      onClickAlbum={() => {}}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
    />
  );
};

export default AlbumsPage;
