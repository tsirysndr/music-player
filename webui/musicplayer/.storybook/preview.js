import { Client as Styletron } from "styletron-engine-atomic";
import { Provider as StyletronProvider } from "styletron-react";
import { LightTheme, BaseProvider } from "baseui";
import { theme } from "../src/Theme";
import "../src/index.css";

const engine = new Styletron();

export const parameters = {
  actions: { argTypesRegex: "^on[A-Z].*" },
  controls: {
    matchers: {
      color: /(background|color)$/i,
      date: /Date$/,
    },
  },
};

export const decorators = [
  (Story) => (
    <StyletronProvider value={engine}>
      <BaseProvider theme={theme}>
        <Story />
      </BaseProvider>
    </StyletronProvider>
  ),
];
