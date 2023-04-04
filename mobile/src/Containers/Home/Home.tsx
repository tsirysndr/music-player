import styled from '@emotion/native';
import React, {FC} from 'react';
import {TouchableWithoutFeedback} from 'react-native';
import Ionicons from 'react-native-vector-icons/Ionicons';
import Albums from '../../Components/Albums';
import Artists from '../../Components/Artists';
import Songs from '../../Components/Songs';
import MiniPlayer from '../../Components/MiniPlayer';

const Container = styled.ScrollView`
  width: 100%;
`;

const Header = styled.View`
  height: 100px;
  flex-direction: row;
  align-items: center;
  padding: 0 20px;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 28px;
  flex: 1;
`;

const Settings = styled.View`
  height: 40px;
  width: 40px;
  align-items: center;
  justify-content: center;
`;

const Home: FC = () => {
  return (
    <>
      <Container showsVerticalScrollIndicator={false}>
        <Header>
          <Title>Library</Title>
          <Settings>
            <TouchableWithoutFeedback onPress={() => {}}>
              <Ionicons name="ios-settings-outline" size={24} color="#bdb9b9" />
            </TouchableWithoutFeedback>
          </Settings>
        </Header>
        <Albums />
        <Artists />
        <Songs />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Home;
