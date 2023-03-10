import { FC, useEffect } from "react";
import ControlBar from "./ControlBar";
import { useGetAlbumQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { useSearch } from "../../Hooks/useSearch";
import { useNavigate, useParams } from "react-router-dom";

const ControlBarWithData: FC = () => {
  const params = useParams();
  const {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    disconnectFromDevice,
    connectToCastDevice,
    disconnectFromCastDevice,
  } = useDevices();
  const { data, loading, refetch } = useGetAlbumQuery({
    variables: {
      id: params.id!,
    },
    fetchPolicy: "network-only",
  });

  useEffect(() => {
    params.id && refetch();
  }, [params.id, refetch]);

  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const {
    play,
    pause,
    next,
    previous,
    nowPlaying,
    nextTracks,
    previousTracks,
    playAlbum,
    playNext,
    playTrackAt,
    removeTrackAt,
    playPlaylist,
  } = usePlayback();
  const { onSearch } = useSearch();
  const album =
    !loading && data
      ? {
          ...data.album,
          tracks: data.album.tracks.map((track, index) => ({
            "#": track.trackNumber,
            id: track.id,
            title: track.title,
            artist: track.artists.map((artist) => artist.name).join(", "),
            time: formatTime(track.duration! * 1000),
            artistId: track.artists[0].id,
            albumId: data.album.id,
            index,
          })),
        }
      : { tracks: [] };
  return (
    <ControlBar
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      nowPlaying={nowPlaying}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      castDevices={castDevices}
      currentCastDevice={currentCastDevice}
      connectToCastDevice={(id) => connectToCastDevice({ variables: { id } })}
      disconnectFromCastDevice={() => disconnectFromCastDevice()}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
    />
  );
};

export default ControlBarWithData;
