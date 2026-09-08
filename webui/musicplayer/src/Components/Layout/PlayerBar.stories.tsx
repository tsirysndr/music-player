import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { noop } from "../../Stories/fixtures";
import PlayerBar from "./PlayerBar";

/**
 * The miniplayer, in every state it can be in.
 *
 * It is presentational — `PlayerBarWithData` supplies the state in the app —
 * so each of these is a plain set of props. Widen the canvas to see the VFD
 * and the volume knob: below `lg` the bar collapses to the transport, and
 * below `sm` it drops the shuffle, repeat and heart too.
 */
const meta: Meta<typeof PlayerBar> = {
  title: "Layout/PlayerBar",
  component: PlayerBar,
  parameters: { layout: "fullscreen" },
  args: {
    title: "End Of The Beginning",
    artist: "Black Sabbath",
    album: "13 (Deluxe Version)",
    progress: 161_000,
    duration: 486_295,
    playing: true,
    volume: 0.75,
    onPlay: noop,
    onPause: noop,
    onNext: noop,
    onPrevious: noop,
    onSeek: noop,
    onToggleLike: noop,
    onVolume: noop,
    onToggleMute: noop,
    onOpenDevices: noop,
    onToggleQueue: noop,
    onOpenFullPlayer: noop,
    onOpenAudioSettings: noop,
  },
};

export default meta;

type Story = StoryObj<typeof PlayerBar>;

export const Playing: Story = {};

export const Paused: Story = {
  args: { playing: false },
};

/**
 * Muted with `m` or the speaker button. The level is kept on the daemon, so
 * the knob stays where it was and the readout says "muted" rather than 0%.
 */
export const Muted: Story = {
  args: { muted: true },
};

/** Nothing loaded: the seek bar is inert and the VFD shows its stop square. */
export const Stopped: Story = {
  args: {
    title: undefined,
    artist: undefined,
    album: undefined,
    stopped: true,
    playing: false,
    progress: 0,
    duration: 0,
  },
};

export const Liked: Story = {
  args: { liked: true },
};

/** With cover art — the artwork slot opens the full player. */
export const WithArtwork: Story = {
  args: {
    cover:
      "https://resources.tidal.com/images/c3672adf/ca16/4db3/a375/64904d1f7d39/320x320.jpg",
  },
};

/**
 * A station: no seek, no skip, the VFD names the source instead of counting,
 * and the heart is the bookmark.
 */
export const Radio: Story = {
  args: {
    isRadio: true,
    title: "Nightride FM",
    artist: undefined,
    album: "Nightride FM",
    duration: 0,
    progress: 0,
  },
};

/** Once ICY metadata names the song, the second line reads "Artist · Station". */
export const RadioWithMetadata: Story = {
  args: {
    isRadio: true,
    title: "Sunset Drive",
    artist: "Timecop1983",
    album: "Nightride FM",
    liked: true,
    duration: 0,
    progress: 0,
  },
};

/** An overlong title scrolls itself rather than being cut off. */
export const LongTitle: Story = {
  args: {
    title:
      "A title long enough that the player bar has to scroll it rather than cut it off",
    artist: "An Artist With A Similarly Unreasonable Name",
  },
};

export const QueueOpen: Story = {
  args: { queueOpen: true },
};

export const AudioSettingsOpen: Story = {
  args: { audioSettingsOpen: true },
};

/** Live: the transport, the seek bar and the volume knob all work. */
export const Interactive: Story = {
  render: function Interactive(args) {
    const [playing, setPlaying] = useState(true);
    const [progress, setProgress] = useState(161_000);
    const [volume, setVolume] = useState(0.75);
    const [muted, setMuted] = useState(false);
    const [liked, setLiked] = useState(false);
    const [queueOpen, setQueueOpen] = useState(false);

    return (
      <PlayerBar
        {...args}
        playing={playing}
        progress={progress}
        volume={volume}
        muted={muted}
        liked={liked}
        queueOpen={queueOpen}
        onPlay={() => setPlaying(true)}
        onPause={() => setPlaying(false)}
        onSeek={setProgress}
        onVolume={(next) => {
          setVolume(next);
          setMuted(false);
        }}
        onToggleMute={() => setMuted((was) => !was)}
        onToggleLike={() => setLiked((was) => !was)}
        onToggleQueue={() => setQueueOpen((open) => !open)}
      />
    );
  },
};
