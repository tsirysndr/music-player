import AlbumDetails from "../../Components/AlbumDetails";
import {
  useCurrentlyPlayingSongQuery,
  useNextMutation,
  usePauseMutation,
  usePlayMutation,
  usePreviousMutation,
} from "../../Hooks/GraphQL";

const AlbumDetailsPage = () => {
  const { data: playback } = useCurrentlyPlayingSongQuery();
  const [play] = usePlayMutation();
  const [pause] = usePauseMutation();
  const [next] = useNextMutation();
  const [previous] = usePreviousMutation();
  return (
    <AlbumDetails
      onBack={() => {}}
      onClickLibraryItem={() => {}}
      onClickAlbum={() => {}}
      onPlay={() => {}}
      onPause={() => {}}
      onNext={() => {}}
      onPrevious={() => {}}
      onShuffle={() => {}}
      onRepeat={() => {}}
      album={{}}
    />
  );
};

export default AlbumDetailsPage;
