import styled from '@emotion/native';
import React from 'react';

import {View} from 'react-native';

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 24px;
`;

const Playlists = () => {
  return (
    <View>
      <Title>Playlists</Title>
    </View>
  );
};

export default Playlists;
