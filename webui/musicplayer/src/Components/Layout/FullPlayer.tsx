import { useEffect } from "react";
import { resourceUriResolver } from "../../ResourceUriResolver";
import { Artwork, IconButton, Icons } from "../UI";

export type FullPlayerProps = {
  open: boolean;
  title?: string;
  cover?: string | null;
  isRadio?: boolean;
  onClose: () => void;
};

/**
 * The desktop's full-window now-playing canvas.
 *
 * It is *only* the artwork, blown up over a blurred copy of itself, with a
 * back button. The title, artist, transport and seek stay where they already
 * are — in the player bar, which the canvas deliberately stops short of and
 * which turns translucent underneath it. That is what the Slint client does
 * (`full-player` is `parent.height - 92px` tall), and duplicating the
 * transport here would leave two of everything on screen.
 *
 * Presentational: `FullPlayerWithData` supplies the state.
 */
const FullPlayer = ({
  open,
  title,
  cover,
  isRadio,
  onClose,
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

      <div className="relative flex h-full items-center justify-center p-[70px]">
        <Artwork
          src={cover}
          alt={title}
          fallbackIcon={isRadio ? Icons.broadcast : Icons.music}
          className="aspect-square h-full max-h-[480px] w-auto max-w-full shadow-[0_22px_70px_rgba(0,0,0,0.66)]"
        />
      </div>
    </div>
  );
};

export default FullPlayer;
