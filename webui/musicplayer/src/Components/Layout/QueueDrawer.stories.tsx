import type { Meta, StoryObj } from "@storybook/react-vite";
import { graphql, HttpResponse } from "msw";
import { Provider, createStore } from "jotai";
import type { ReactNode } from "react";
import { nowPlayingAtom, queueOpenAtom, type NowPlaying } from "../../State";
import QueueDrawer from "./QueueDrawer";

const TIDAL = "https://resources.tidal.com/images";
const COVER = `${TIDAL}/f87d9afc/075e/43f4/bbbc/7770b46cb8aa/320x320.jpg`;

const NOW_PLAYING: NowPlaying = {
  id: "t0",
  title: "End Of The Beginning",
  artist: "Black Sabbath",
  album: "13 (Deluxe Version)",
  cover: COVER,
  duration: 486_295,
  progress: 121_000,
  isPlaying: true,
};

/**
 * The drawer renders nothing unless the queue is open, and `nowPlayingAtom` is
 * filled by `AppStateSync` in the running app — so a story seeds the store
 * directly rather than mounting the shell to reach one card.
 */
const withStore = (nowPlaying?: NowPlaying) => (Story: () => ReactNode) => {
  const store = createStore();
  store.set(queueOpenAtom, true);
  if (nowPlaying) store.set(nowPlayingAtom, nowPlaying);
  return (
    <Provider store={store}>
      <div className="flex h-dvh justify-end bg-window">
        <Story />
      </div>
    </Provider>
  );
};

/** A track shaped as the tracklist query returns it. */
const track = (id: string, title: string, duration: number) => ({
  __typename: "Track",
  id,
  title,
  artist: "Black Sabbath",
  duration,
  artists: [{ __typename: "Artist", id: "a1", name: "Black Sabbath" }],
  album: {
    __typename: "Album",
    id: "3b16b0caed8f12736ae5debf7363fc79",
    title: "13 (Deluxe Version)",
    cover: null,
  },
});

const tracklist = (next: unknown[], previous: unknown[]) => [
  graphql.query("GetTracklist", () =>
    HttpResponse.json({
      data: {
        tracklistTracks: {
          __typename: "Tracklist",
          nextTracks: next,
          previousTracks: previous,
        },
      },
    })
  ),
];

const NEXT = [
  track("t1", "God Is Dead?", 532.317),
  track("t2", "Loner", 300.001),
  track("t3", "Zeitgeist", 266.44),
  track("t4", "Age Of Reason", 424.6),
];
const PREVIOUS = [track("t0", "End Of The Beginning", 486.295)];

/**
 * The desktop's queue drawer: a now-playing card, what is up next, and the
 * history behind a second tab.
 *
 * On a phone it becomes a full-height sheet rather than a 320px rail.
 */
const meta: Meta<typeof QueueDrawer> = {
  title: "Layout/QueueDrawer",
  component: QueueDrawer,
  parameters: { msw: tracklist(NEXT, PREVIOUS) },
  decorators: [withStore(NOW_PLAYING)],
};

export default meta;

type Story = StoryObj<typeof QueueDrawer>;

export const Default: Story = {};

/** Nothing queued behind the current track. */
export const EmptyQueue: Story = {
  parameters: { msw: tracklist([], PREVIOUS) },
};

/** Opened before anything has played: no card, no rows. */
export const NothingPlaying: Story = {
  parameters: { msw: tracklist([], []) },
  decorators: [withStore()],
};

/** A radio stream — no album, and the duration reads as a dash. */
export const Radio: Story = {
  parameters: { msw: tracklist([], []) },
  decorators: [
    withStore({
      title: "Radio Paradise",
      artist: "Mellow Mix",
      duration: 0,
      progress: 0,
      isPlaying: true,
    }),
  ],
};

/** Long enough to scroll, which is what the drawer is usually doing. */
export const LongQueue: Story = {
  parameters: {
    msw: tracklist(
      Array.from({ length: 30 }, (_, index) =>
        track(`q${index}`, `Queued track ${index + 1}`, 180 + index)
      ),
      PREVIOUS
    ),
  },
};
