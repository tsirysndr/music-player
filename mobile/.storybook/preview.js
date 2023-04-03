import styled from '@emotion/native';

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
    <Container>
      <Story />
    </Container>
  ),
];
