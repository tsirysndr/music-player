import React, {FC} from 'react';
import Progressbar from '../Progressbar';
import styled from '@emotion/native';
import Ionicons from 'react-native-vector-icons/Ionicons';

const Container = styled.View`
  flex: 1;
  width: 93%;
`;

const Time = styled.Text`
  font-family: 'Gilroy-Medium';
  font-size: 12px;
`;

const Duration = styled.Text`
  font-family: 'Gilroy-Medium';
  font-size: 12px;
`;

const TimeContainer = styled.View`
  flex-direction: row;
  justify-content: space-between;
  margin-left: 14px;
  margin-right: 14px;
`;

const Button = styled.TouchableOpacity`
  justify-content: center;
  align-items: center;
`;

const Row = styled.View`
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  height: 140px;
  margin-left: 14px;
  margin-right: 14px;
`;

export type PlayerControlsProps = {};

const PlayerControls: FC<PlayerControlsProps> = () => {
  return (
    <Container>
      <Progressbar
        progress={30}
        onValueChange={() => {}}
        thumbTintColor="#fff"
      />
      <TimeContainer>
        <Time>01:13</Time>
        <Duration>3:16</Duration>
      </TimeContainer>
      <Row>
        <Button>
          <Ionicons name="ios-shuffle" size={24} color="#fff" />
        </Button>
        <Button>
          <Ionicons name="ios-play-skip-back" size={28} color="#fff" />
        </Button>
        <Button>
          <Ionicons name="ios-play" size={46} color="#fff" />
        </Button>
        <Button>
          <Ionicons name="ios-play-skip-forward" size={28} color="#fff" />
        </Button>
        <Button>
          <Ionicons name="md-repeat" size={24} color="#fff" />
        </Button>
      </Row>
    </Container>
  );
};

export default PlayerControls;
