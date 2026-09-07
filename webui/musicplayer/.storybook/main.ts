import type { StorybookConfig } from "@storybook/react-vite";

/**
 * Storybook on the Vite builder, sharing `vite.config.ts` — the webpack5
 * builder and the create-react-app preset went with react-scripts, and a
 * second bundler config would drift from the app's.
 */
const config: StorybookConfig = {
  stories: ["../src/**/*.stories.@(js|jsx|ts|tsx)"],
  staticDirs: ["../public"],
  // `msw-storybook-addon` answers the GraphQL calls with the same handlers
  // the test suite uses, so a data-connected component renders real-shaped
  // data in Storybook too.
  addons: ["@storybook/addon-links", "msw-storybook-addon"],
  framework: {
    name: "@storybook/react-vite",
    options: {},
  },
};

export default config;
