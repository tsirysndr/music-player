import styled from '@emotion/native';
import {MockedProvider} from '@apollo/client/testing';
import {RecoilRoot} from 'recoil';

export const parameters = {
  controls: {
    matchers: {
      color: /(background|color)$/i,
      date: /Date$/,
    },
  },
};

const Container = styled.View`
  flex: 1;
  align-items: center;
  justify-content: center;
  background-color: #000;
`;

export const decorators = [
  Story => (
    <MockedProvider mocks={[]} addTypename={true}>
      <RecoilRoot>
        <Container>
          <Story />
        </Container>
      </RecoilRoot>
    </MockedProvider>
  ),
];
