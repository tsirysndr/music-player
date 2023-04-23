import styled from '@emotion/native';
import React, {FC} from 'react';
import CurrentTrack from '../../Components/CurrentTrack';
import {BlurView as DefaultBlurView} from '@react-native-community/blur';
import PlayerControls from '../../Components/PlayerControls';
import {useCover} from '../../Hooks/useCover';

const Background = styled.ImageBackground`
  flex: 1;
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  width: 100%;
`;

const NoBackground = styled.View`
  flex: 1;
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  width: 100%;
  background-color: #000;
`;

const BlurView = styled(DefaultBlurView)`
  flex: 1;
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
`;

const Header = styled.View`
  height: 100px;
`;

export type PlayerProps = {
  track: any;
};

const Player: FC<PlayerProps> = ({track}) => {
  const cover = useCover(track?.cover);
  return (
    <>
      {track?.cover && (
        <>
          <Background
            source={{
              uri: cover,
            }}
          />
          <BlurView
            blurType="dark"
            blurAmount={32}
            reducedTransparencyFallbackColor="white"
          />
        </>
      )}
      {!track?.cover && <NoBackground />}
      <Header />
      <CurrentTrack />
      <PlayerControls />
    </>
  );
};

export default Player;
