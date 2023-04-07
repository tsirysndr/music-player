import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';

const Container = styled.View`
  flex: 1;
`;

export type SettingsProps = {
  onGoBack: () => void;
};

const Settings: FC<SettingsProps> = props => {
  const {onGoBack} = props;
  return (
    <>
      <Container>
        <Header title="Settings" onGoBack={onGoBack} />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Settings;
