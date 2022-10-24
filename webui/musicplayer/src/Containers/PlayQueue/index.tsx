import { useNavigate } from "react-router-dom";
import PlayQueue from "../../Components/PlayQueue";

const PlayQueuePage = () => {
  const navigate = useNavigate();
  return (
    <PlayQueue
      tracks={[]}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
    />
  );
};

export default PlayQueuePage;
