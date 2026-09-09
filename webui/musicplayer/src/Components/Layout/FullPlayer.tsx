import { useEffect, useState } from "react";
import { resourceUriResolver } from "../../ResourceUriResolver";
import { Artwork, cn, EqualizerBars, IconButton, Icons, Waveform } from "../UI";

export type FullPlayerProps = {
  open: boolean;
  title?: string;
  cover?: string | null;
  isRadio?: boolean;
  onClose: () => void;
  /** Peak per bar from the daemon's analysis. Empty until it has one. */
  waveform?: number[];
  /** How far through the track, 0..1. */
  progress?: number;
  /** Seconds, for turning a click on the waveform into a position. */
  duration?: number;
  onSeek?: (seconds: number) => void;
  /** Output levels, for the bars. */
  levels?: { left: number; right: number };
  isPlaying?: boolean;
};

/**
 * The desktop's full-window now-playing canvas.
 *
 * The artwork blown up over a blurred copy of itself, with the track's waveform
 * and a live equalizer under it. The title, artist and transport stay in the
 * player bar, which the canvas stops short of and which turns translucent
 * underneath it — duplicating the transport here would leave two of everything
 * on screen.
 *
 * The waveform *is* a second seek control, and deliberately so: it is the one
 * thing a progress bar cannot do, because it shows where in the song you are
 * aiming rather than what percentage.
 *
 * Presentational: `FullPlayerWithData` supplies the state.
 */
const FullPlayer = ({
  open,
  title,
  cover,
  isRadio,
  onClose,
  waveform = [],
  progress = 0,
  duration = 0,
  onSeek,
  levels,
  isPlaying = true,
}: FullPlayerProps) => {
  // On by default — it is most of the point of the full player — with `v` for
  // anyone who wants the artwork alone.
  const [showEqualizer, setShowEqualizer] = useState(true);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
      // Not while typing: the command palette and the filter boxes are real
      // text inputs, and a bare letter must reach them rather than the canvas.
      const target = event.target as HTMLElement | null;
      if (target?.closest("input, textarea, [contenteditable]")) return;
      if (event.key === "v" && !event.metaKey && !event.ctrlKey) {
        setShowEqualizer((shown) => !shown);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open || !title) return null;

  const resolved = resourceUriResolver.resolve(cover ?? undefined);

  return (
    <div
      // Stops above the player bar — 76px on a phone, 92px from `lg` — so the
      // miniplayer stays visible and keeps the transport.
      className="fixed inset-x-0 top-0 z-40 overflow-hidden bg-window"
      style={{ bottom: "var(--player-bar-height)" }}
    >
      {resolved && (
        <div
          style={{ backgroundImage: `url("${resolved}")` }}
          className="absolute -inset-10 scale-110 bg-cover bg-center opacity-[0.48] blur-3xl"
        />
      )}
      {/* The scrim over the blur, so the art below reads without drowning
          everything on top of it.

          The window colour rather than a fixed near-black: this canvas is full
          of themed controls — the waveform, the back button, the equalizer —
          and a dark scrim under a light skin left every one of them dark on
          dark. Tinting with the skin's own background keeps the surface a
          shade of what the theme expects, whichever skin is on. */}
      <div className="absolute inset-0 bg-window/[0.78]" />

      <IconButton
        icon={Icons.chevronLeft}
        iconSize={20}
        size={42}
        aria-label="Close full player"
        onClick={onClose}
        className="absolute left-[22px] top-[22px] z-10"
      />

      <div
        className={cn(
          "relative flex h-full flex-col items-center justify-center gap-7 px-[70px] pt-[54px]",
          // Reclaims the equalizer's strip when it is hidden, so the artwork
          // grows rather than leaving a gap where it was.
          showEqualizer ? "pb-[86px]" : "pb-[54px]"
        )}
      >
        <Artwork
          src={cover}
          alt={title}
          fallbackIcon={isRadio ? Icons.broadcast : Icons.music}
          className="aspect-square min-h-0 flex-1 shrink w-auto max-w-full shadow-[0_22px_70px_rgba(0,0,0,0.66)]"
        />

        {/* Under the art, at the width of the canvas rather than the artwork:
            a waveform is a timeline, and cropping it to a square would make
            the same track look different on a different screen. */}
        <div className="flex w-full max-w-[760px] shrink-0 flex-col gap-3">
          {/* A live stream has no length, so it has no waveform and no
              position to seek to — the bars alone are the honest display. */}
          {!isRadio && (
            <div className="h-[52px] w-full">
              <Waveform
                bars={waveform}
                progress={progress}
                duration={duration}
                onSeek={onSeek}
              />
            </div>
          )}
        </div>
      </div>

      {/* Full-bleed along the bottom edge, flush against the player bar rather
          than boxed into the column above. The equalizer is the audio leaving
          the machine, not a property of the track — so it spans the window,
          and the waveform, which *is* the track, stays with the artwork. */}
      {showEqualizer && (
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-[64px]">
          <EqualizerBars
            left={levels?.left ?? 0}
            right={levels?.right ?? 0}
            playing={isPlaying}
            bars={96}
          />
        </div>
      )}
    </div>
  );
};

export default FullPlayer;
