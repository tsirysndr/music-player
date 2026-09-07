# Tests

[Vitest](https://vitest.dev) on jsdom, with
[Testing Library](https://testing-library.com) and
[MSW](https://mswjs.io) for the GraphQL layer.

```sh
bun run test           # once
bun run test:watch     # watch
bun run test:coverage   # with an lcov report
```

## Layout

| File           | What it is                                                    |
| -------------- | ------------------------------------------------------------- |
| `setup.ts`     | Runs before every file: starts MSW, adds the jsdom stubs       |
| `server.ts`    | The MSW server, so a test can narrow one operation             |
| `handlers.ts`  | The default GraphQL responses, keyed by operation name         |
| `fixtures.ts`  | Generated from a real library — see below                      |
| `render.tsx`   | `renderWithProviders` / `renderWithRouter`, plus `userEvent`   |

`onUnhandledRequest: "error"` is deliberate: a request nobody stubbed is
almost always a test asserting against a loading state it did not mean to be
in. Add the operation to `handlers.ts` if every test needs it, or narrow it in
one test with `server.use(graphql.query("GetAlbums", …))`.

## Fixtures come from a real library

`fixtures.ts` is generated from an actual music-player database rather than
written by hand. Ids are the 32-character md5 hashes the scanner produces,
durations are fractional seconds, and titles carry the parenthetical suffixes,
punctuation and stray whitespace that turn up in real tag data — all things a
hand-written fixture quietly rounds off, and all things the formatting and
elision code has to survive. (One title in this library has a leading space,
which is why `TracksWithData.test.tsx` normalizes before matching.)

Regenerate against your own library with:

```sh
python3 - "$HOME/Library/Application Support/music-player/music-player.sqlite3" \
  > src/test/fixtures.ts <<'PY'
import json, sqlite3, sys
db = sqlite3.connect(sys.argv[1]); db.row_factory = sqlite3.Row
# …see the git history of this file for the full script…
PY
```

The shape is what the GraphQL documents in `src/GraphQL/` select, so a change
there means regenerating.

## Conventions

- **Query by role and accessible name.** If a query is awkward, that is usually
  the component missing a label rather than the test needing a `data-testid` —
  several `aria-label`s in the design system exist because a test could not
  find the element otherwise.
- **Scope to `<main>`** when the sidebar renders the same names as the page:
  the recent-playlists list and the playlists page both show every playlist.
- **Seed atoms directly** for state that `AppStateSync` fills in the running
  app. Mounting the whole shell to reach one card tests the shell instead.
- **A fresh `QueryClient` per render**, with retries off — `render.tsx` does
  this. A shared client leaks one test's cached data into the next.
