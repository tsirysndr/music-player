import React, {useMemo} from 'react';
import CurrentTrack from './CurrentTrack';
import {useRecoilState} from 'recoil';
import {currentTrackState} from './CurrentTrackState';
import {playQueueState} from '../PlayQueue/PlayQueueState';
import {contextMenuState} from '../ContextMenu/ContextMenuState';

const CurrentTrackWithData = () => {
  const [contextMenu, setContextMenu] = useRecoilState(contextMenuState);
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  const [{previousTracks, nextTracks, position}, setPlayQueue] =
    useRecoilState(playQueueState);
  const tracks = [...previousTracks, ...nextTracks];
  const initialPage = useMemo(() => position, [position]);

  const onPageSelected = (e: any) => {
    setPlayQueue({
      nextTracks: tracks.slice(e.nativeEvent.position + 1),
      previousTracks: tracks.slice(0, e.nativeEvent.position + 1),
      position: e.nativeEvent.position,
    });
    setCurrentTrack(tracks[e.nativeEvent.position]);
  };

  const onLike = () => {
    setCurrentTrack({
      ...currentTrack!,
      liked: !currentTrack!.liked,
    });
  };

  const onContextMenu = () => {
    setContextMenu({
      ...contextMenu,
      visible: true,
      enablePlayNext: false,
      type: 'track',
      item: currentTrack,
    });
  };

  return (
    <CurrentTrack
      track={currentTrack}
      initialPage={initialPage}
      onPageSelected={onPageSelected}
      onContextMenu={onContextMenu}
      tracks={tracks}
      onLike={onLike}
    />
  );
};

export default CurrentTrackWithData;
