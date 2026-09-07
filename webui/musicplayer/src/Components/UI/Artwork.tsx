import { useEffect, useState } from "react";
import { resourceUriResolver } from "../../ResourceUriResolver";
import cn from "./cn";
import { Icons, type IconComponent } from "./icons";

export type ArtworkProps = {
  src?: string | null;
  alt?: string;
  /** Glyph drawn when there is no art — a disc for albums, a mic for artists. */
  fallbackIcon?: IconComponent;
  /**
   * Fallback glyph size in px. Left out, it scales with the box — a disc on a
   * 200px card and a disc on a 28px palette row want very different numbers,
   * and a fixed one is always wrong for one of them.
   */
  iconSize?: number;
  /** Artist art is round in the desktop client; album art is square. */
  rounded?: "square" | "skin" | "full";
  className?: string;
};

const RADIUS = {
  square: "",
  skin: "rounded-skin",
  full: "rounded-full",
} as const;

/**
 * Cover art with the skin's placeholder underneath.
 *
 * The placeholder is not swapped in on error alone — it is what shows until an
 * image actually decodes, so a slow or missing cover leaves the skin's colour
 * rather than a broken-image glyph, which is how the desktop's
 * `has-art` flag behaves.
 */
const Artwork = ({
  src,
  alt = "",
  fallbackIcon: Fallback = Icons.disc,
  iconSize,
  rounded = "square",
  className,
}: ArtworkProps) => {
  const resolved = resourceUriResolver.resolve(src ?? undefined);
  const [failed, setFailed] = useState(false);

  useEffect(() => setFailed(false), [resolved]);

  return (
    <div
      className={cn(
        "relative grid shrink-0 place-items-center overflow-hidden bg-art",
        RADIUS[rounded],
        className
      )}
    >
      {resolved && !failed ? (
        <img
          src={resolved}
          alt={alt}
          loading="lazy"
          onError={() => setFailed(true)}
          className="size-full object-cover"
        />
      ) : (
        // CSS wins over the SVG's width/height attributes, so the class is
        // what sizes it when no explicit `iconSize` is given.
        <Fallback
          size={iconSize}
          className={cn("text-muted", iconSize === undefined && "size-[38%]")}
          aria-hidden="true"
        />
      )}
    </div>
  );
};

export default Artwork;
