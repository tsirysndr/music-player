import React, {FC} from 'react';
import TrackRow from './TrackRow';
import {useRecoilState} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';
import {Track} from '../../Types';
import {playQueueState} from '../PlayQueue/PlayQueueState';
import {contextMenuState} from '../ContextMenu/ContextMenuState';

export type TrackRowWithDataProps = {
  track: Track;
  showAlbum?: boolean;
};

const TrackRowWithData: FC<TrackRowWithDataProps> = ({track, showAlbum}) => {
  const [contextMenu, setContextMenu] = useRecoilState(contextMenuState);
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  const [playQueue, setPlayQueue] = useRecoilState(playQueueState);
  const onPlay = (item: Track) => {
    setCurrentTrack(item);
    setPlayQueue({
      ...playQueue,
      previousTracks: playQueue.previousTracks.concat(item),
      position: playQueue.previousTracks.length,
    });
  };
  const onContextMenu = (item: Track) => {
    setContextMenu({
      ...contextMenu,
      visible: true,
      type: 'track',
      item,
      enablePlayNext: true,
    });
  };
  return (
    <TrackRow
      track={track}
      currentTrack={currentTrack}
      onPlay={onPlay}
      showAlbum={showAlbum}
      onPressContextMenu={onContextMenu}
    />
  );
};

export default TrackRowWithData;
