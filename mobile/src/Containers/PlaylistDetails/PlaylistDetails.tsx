import styled from '@emotion/native';
import React, {FC} from 'react';

import {View} from 'react-native';

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 24px;
`;

const PlaylistDetails: FC = () => {
  return (
    <View>
      <Title>PlaylistDetails</Title>
    </View>
  );
};

export default PlaylistDetails;
