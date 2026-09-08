import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { noop } from "../../Stories/fixtures";
import FullPlayer from "./FullPlayer";
import PlayerBar from "./PlayerBar";

const COVER =
  "https://resources.tidal.com/images/c3672adf/ca16/4db3/a375/64904d1f7d39/320x320.jpg";

/**
 * The fullscreen now-playing canvas — the desktop's full-window player, opened
 * from the miniplayer artwork or with `f`.
 *
 * It is *only* the artwork over a blurred copy of itself. The title, transport
 * and seek stay in the player bar, which the canvas stops short of and which
 * goes translucent underneath — the "Over the player bar" story shows the two
 * together, which is how it is actually seen.
 */
const meta: Meta<typeof FullPlayer> = {
  title: "Layout/FullPlayer",
  component: FullPlayer,
  parameters: { layout: "fullscreen" },
  args: {
    open: true,
    title: "End Of The Beginning",
    cover: COVER,
    onClose: noop,
  },
  decorators: [
    (Story) => (
      <div className="h-dvh bg-window">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj<typeof FullPlayer>;

export const Default: Story = {};

/** No cover: the skin's art placeholder and a note glyph. */
export const WithoutArtwork: Story = { args: { cover: undefined } };

/** A station with no logo gets the broadcast glyph instead. */
export const Radio: Story = {
  args: { isRadio: true, title: "Nightride FM", cover: undefined },
};

export const RadioWithLogo: Story = {
  args: { isRadio: true, title: "Nightride FM", cover: COVER },
};

/** Nothing playing: it renders nothing at all rather than an empty canvas. */
export const NothingPlaying: Story = { args: { title: undefined } };

export const Closed: Story = { args: { open: false } };

/**
 * What it actually looks like: the canvas above, the translucent miniplayer
 * below, sharing one blurred backdrop.
 */
export const OverThePlayerBar: Story = {
  render: function OverThePlayerBar(args) {
    const [open, setOpen] = useState(true);
    const [playing, setPlaying] = useState(true);
    const [progress, setProgress] = useState(161_000);

    return (
      <div className="flex h-dvh flex-col bg-window">
        <div className="flex-1" />
        <FullPlayer {...args} open={open} onClose={() => setOpen(false)} />
        <PlayerBar
          title="End Of The Beginning"
          artist="Black Sabbath"
          album="13 (Deluxe Version)"
          cover={COVER}
          playing={playing}
          progress={progress}
          duration={486_295}
          volume={0.75}
          overlay={open}
          onPlay={() => setPlaying(true)}
          onPause={() => setPlaying(false)}
          onNext={noop}
          onPrevious={noop}
          onSeek={setProgress}
          onToggleLike={noop}
          onVolume={noop}
          onToggleMute={noop}
          onOpenDevices={noop}
          onToggleQueue={noop}
          onOpenFullPlayer={() => setOpen(true)}
          onOpenAudioSettings={noop}
        />
      </div>
    );
  },
};
