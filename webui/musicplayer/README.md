# Music Player Web UI

This project was bootstrapped with [Create React App](https://github.com/facebook/create-react-app).

## Tauri

This directory is also configured as a [Tauri](tauri.app/) project. Refer to https://tauri.app/v1/guides/getting-started/prerequisites for system level prerequisites. To run the application in dev mode:

```sh
cargo install tauri-cli
cargo tauri dev
```

## Available Scripts

In the project directory, you can run:

### `npm start`

Runs the app in the development mode.\
Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

The page will reload if you make edits.\
You will also see any lint errors in the console.

### `npm test`

Launches the test runner in the interactive watch mode.\
See the section about [running tests](https://facebook.github.io/create-react-app/docs/running-tests) for more information.

### `npm run build`

Builds the app for production to the `build` folder.\
It correctly bundles React in production mode and optimizes the build for the best performance.

The build is minified and the filenames include the hashes.\
Your app is ready to be deployed!

See the section about [deployment](https://facebook.github.io/create-react-app/docs/deployment) for more information.

### `npm run eject`

**Note: this is a one-way operation. Once you `eject`, you can’t go back!**

If you aren’t satisfied with the build tool and configuration choices, you can `eject` at any time. This command will remove the single build dependency from your project.

Instead, it will copy all the configuration files and the transitive dependencies (webpack, Babel, ESLint, etc) right into your project so you have full control over them. All of the commands except `eject` will still work, but they will point to the copied scripts so you can tweak them. At this point you’re on your own.

You don’t have to ever use `eject`. The curated feature set is suitable for small and middle deployments, and you shouldn’t feel obligated to use this feature. However we understand that this tool wouldn’t be useful if you couldn’t customize it when you are ready for it.

## Learn More

You can learn more in the [Create React App documentation](https://facebook.github.io/create-react-app/docs/getting-started).

To learn React, check out the [React documentation](https://reactjs.org/).

## Development

The build moved from create-react-app to [Vite](https://vite.dev) in September
2026. Two consumers pin the output shape, so both are set explicitly in
`vite.config.ts`:

- `webui/src/lib.rs` embeds `musicplayer/build/` with `rust-embed`, so the
  output directory is `build/`, not Vite's default `dist/`.
- `src-tauri/tauri.conf.json` sets `devUrl: "http://localhost:3000"`, so the
  dev server uses port 3000 with `strictPort` — silently moving to 3001 would
  leave Tauri pointing at nothing.

```sh
bun install
bun run dev              # http://localhost:3000
bun run build            # typecheck, then bundle into build/
bun run storybook        # component workshop on :6007
bun run graphql:generate # regenerate typed hooks (see below)
```

Environment variables use Vite's `VITE_` prefix and `import.meta.env`, not
`REACT_APP_` / `process.env`:

| Variable              | Effect                                    |
| --------------------- | ----------------------------------------- |
| `VITE_NATIVE_WRAPPER` | Set to `tauri` when building inside Tauri |
| `VITE_API_URL`        | GraphQL endpoint in development           |

### Regenerating the GraphQL hooks

`src/Hooks/GraphQL.tsx` is generated from `graphql.schema.json`. Refresh that
from the Rust schema — no running daemon needed, and it matches the working
tree rather than whatever binary happens to be listening:

```sh
cargo run --release -p music-player-graphql --example dump_schema \
    -- webui/musicplayer/graphql.schema.json
bun run graphql:generate
```
