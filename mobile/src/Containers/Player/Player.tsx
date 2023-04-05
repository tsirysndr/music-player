import styled from '@emotion/native';
import React, {FC} from 'react';
import CurrentTrack from '../../Components/CurrentTrack';
import {BlurView as DefaultBlurView} from '@react-native-community/blur';
import {tracks} from '../../Mocks/Tracks';
import PlayerControls from '../../Components/PlayerControls';

const Background = styled.ImageBackground`
  flex: 1;
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  width: 100%;
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

const Player: FC = () => {
  return (
    <>
      <Background
        source={{
          uri: tracks[0].cover,
        }}
      />
      <BlurView
        blurType="dark"
        blurAmount={32}
        reducedTransparencyFallbackColor="white"
      />
      <Header />
      <CurrentTrack />
      <PlayerControls />
    </>
  );
};

export default Player;
