# Music Player Web UI

The browser client, served by the daemon.
desktop wrapper loads.

## Design system

The UI is a port of the Slint desktop client in `desktop/`. The two are meant
to be the same product, so the port is literal rather than "inspired by":

- **Skins.** `src/styles/skins.css` holds the five skins from
  `desktop/skins/*.toml` — Synthwave, Late Night, Neutron, Lunar, Porcelain —
  as CSS custom properties under `[data-skin="…"]`. The names match the Slint
  `Theme` global's minus the prefix: `window_bg` → `--window-bg`. Adding a skin
  means adding one block here and one `.toml` there; anything that drifts stops
  the two clients from looking alike, which is the whole point.
- **Tokens.** `src/index.css` maps those onto Tailwind theme colours, so
  `bg-panel` here and `Theme.panel-bg` there are the same value. Components
  never hard-code a colour.
- **Fonts.** Roboto Mono Variable for the interface and JetBrains Mono for the
  places that read as a readout rather than as prose — the VFD, timecodes, ids,
  the filter editor. The desktop client embeds the same two.
- **Components.** `src/Components/UI/` is a React port of
  `desktop/ui/components.slint`, one file per component and the same names:
  `IconButton`, `PlayPauseButton`, `SlideBar`, `Knob`, `VfdDisplay`,
  `MeterStrip`, `MarqueeText`, `TrackRow`, `AlbumCard`, `RadioRow`, and so on.
- **Icons.** `src/Components/UI/icons.tsx` maps the desktop's icon names onto
  [Tabler](https://tabler.io/icons); a component ported from Slint asks for the
  same icon here as it does there.
- **Layout.** `src/Components/Layout/AppShell.tsx` reproduces the desktop
  window: sidebar, 62px header, content, 92px player bar, queue drawer.

The stack is [Tailwind CSS v4](https://tailwindcss.com) with
[HeroUI](https://heroui.com) for the overlay primitives (modal, popover, toast
— React Aria underneath, so focus trapping and dismissal are handled). Base
Web, Styletron, styled-components and Emotion are gone.

### Responsive

Below `lg` the sidebar is replaced by a bottom tab bar (`Layout/BottomTabs`)
with the overflow sections behind a "More" sheet; the player bar drops the VFD
readout and the volume knob, and the queue becomes a full-height sheet rather
than a fixed rail.

### Forms

Every form is [react-hook-form](https://react-hook-form.com) with a
[zod](https://zod.dev) resolver. `TextField`/`TextAreaField`/`Select` take an
`error` prop and render the message themselves, so a form is
`register(...)` plus `errors.field?.message`.


## Development

The build moved from create-react-app to [Vite](https://vite.dev) in September
2026. Two consumers pin the output shape, so both are set explicitly in
`vite.config.ts`:

- `webui/src/lib.rs` embeds `musicplayer/build/` with `rust-embed`, so the
  output directory is `build/`, not Vite's default `dist/`.

```sh
bun install
bun run dev              # http://localhost:3000
bun run build            # typecheck, then bundle into build/
bun run test             # vitest
bun run test:coverage    # …with an lcov report
bun run storybook        # component workshop on :6007
bun run graphql:generate # regenerate typed hooks (see below)
```

## Tests

Vitest on jsdom, with Testing Library and MSW for the GraphQL layer. The
fixtures are generated from a real music-player library rather than written by
hand, so the tests see the ids, fractional durations and untidy tag data the
app actually gets. See [`src/test/README.md`](src/test/README.md).

The `webui tests` GitHub workflow runs the suite, the typecheck and both
builds on any push or PR that touches this directory.

Storybook has a **Skin** picker in the toolbar that writes the same
`data-skin` attribute the app writes, so any story can be checked against all
five skins.

Environment variables use Vite's `VITE_` prefix and `import.meta.env`, not
`REACT_APP_` / `process.env`:

| Variable              | Effect                                    |
| --------------------- | ----------------------------------------- |
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
