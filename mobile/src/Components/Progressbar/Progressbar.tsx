import React, {FC} from 'react';
import DefaultSlider from '@react-native-community/slider';
import styled from '@emotion/native';

const Container = styled.View`
  width: 100%;
`;

const Slider = styled(DefaultSlider)`
  width: 100%;
`;

export type ProgressbarProps = {
  progress: number;
  onValueChange: (value: number) => void;
  thumbTintColor?: string;
};

const Progressbar: FC<ProgressbarProps> = ({
  progress,
  thumbTintColor,
  onValueChange,
}) => {
  return (
    <Container>
      <Slider
        minimumValue={0}
        maximumValue={100}
        minimumTrackTintColor="#ab28fc"
        maximumTrackTintColor="#5f5f5f"
        thumbTintColor={thumbTintColor}
        value={progress}
        onValueChange={onValueChange}
      />
    </Container>
  );
};

Progressbar.defaultProps = {
  thumbTintColor: '#ab28fc',
};

export default Progressbar;
