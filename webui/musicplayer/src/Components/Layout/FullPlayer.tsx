import { useEffect } from "react";
import { resourceUriResolver } from "../../ResourceUriResolver";
import { Artwork, EqualizerBars, IconButton, Icons, Waveform } from "../UI";

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
  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
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
      {/* The scrim the desktop lays over the blur, so the art below reads. */}
      <div className="absolute inset-0 bg-[#08060d]/[0.73]" />

      <IconButton
        icon={Icons.chevronLeft}
        iconSize={20}
        size={42}
        aria-label="Close full player"
        onClick={onClose}
        className="absolute left-[22px] top-[22px] z-10"
      />

      <div className="relative flex h-full flex-col items-center justify-center gap-7 px-[70px] py-[54px]">
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
          <div className="h-[34px] w-full opacity-90">
            <EqualizerBars
              left={levels?.left ?? 0}
              right={levels?.right ?? 0}
              playing={isPlaying}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default FullPlayer;
