import cn from "./cn";
import { Icons } from "./icons";

export type LikeButtonProps = {
  liked?: boolean;
  iconSize?: number;
  size?: number;
  /** Dim until hovered — how the desktop treats the heart in a track row. */
  subtle?: boolean;
  className?: string;
  onClick: () => void;
};

/** The desktop's `LikeButton`: a filled heart when liked, an outline when not. */
const LikeButton = ({
  liked,
  iconSize = 18,
  size = 30,
  subtle,
  className,
  onClick,
}: LikeButtonProps) => {
  const Icon = liked ? Icons.heart : Icons.heartOutline;
  return (
    <button
      type="button"
      aria-label={liked ? "Remove from liked" : "Add to liked"}
      aria-pressed={!!liked}
      onClick={onClick}
      style={{ width: size, height: size }}
      className={cn(
        "group grid shrink-0 place-items-center rounded-full transition-all hover:bg-hover",
        subtle && !liked && "opacity-35 group-hover/row:opacity-100",
        className
      )}
    >
      <Icon
        size={iconSize}
        className={cn(
          "transition-colors",
          liked ? "text-accent" : "text-dim group-hover:text-fg"
        )}
      />
    </button>
  );
};

export default LikeButton;
