import Artwork from "./Artwork";
import cn from "./cn";
import { Icons, type IconComponent } from "./icons";
import LikeButton from "./LikeButton";

export type StationItem = {
  id: string;
  name: string;
  subtitle?: string;
  source?: string;
  logo?: string | null;
  bookmarked?: boolean;
};

export type RadioRowProps = {
  station: StationItem;
  playing?: boolean;
  onPlay: () => void;
  onBookmark?: () => void;
};

/** The desktop's `RadioRow`: station logo, name, genre · source, bookmark. */
export const RadioRow = ({
  station,
  playing,
  onPlay,
  onBookmark,
}: RadioRowProps) => (
  <div
    className={cn(
      "group flex h-[58px] items-center gap-3 rounded-control px-3",
      playing ? "bg-selected" : "hover:bg-hover"
    )}
  >
    <button
      type="button"
      onClick={onPlay}
      className="flex min-w-0 flex-1 items-center gap-3 text-left"
    >
      <Artwork
        src={station.logo}
        alt={station.name}
        fallbackIcon={Icons.broadcast}
        iconSize={19}
        rounded="skin"
        className="size-10 [&_svg]:text-accent"
      />
      <span className="flex min-w-0 flex-col gap-[2px]">
        <span
          className={cn(
            "truncate text-[13px] font-semibold",
            playing ? "text-accent" : "text-fg"
          )}
        >
          {station.name}
        </span>
        <span className="truncate text-[10px] text-muted">
          {[station.subtitle, station.source].filter(Boolean).join("  ·  ")}
        </span>
      </span>
    </button>
    {onBookmark && (
      <LikeButton
        liked={station.bookmarked}
        iconSize={18}
        onClick={onBookmark}
      />
    )}
  </div>
);

export type RadioCategoryTileProps = {
  label: string;
  term: string;
  icon: IconComponent;
  /** The desktop gives each category its own hue; passed straight through. */
  color: string;
  onSelect: (label: string, term: string) => void;
};

/** The desktop's `RadioCategoryTile`. */
export const RadioCategoryTile = ({
  label,
  term,
  icon: Icon,
  color,
  onSelect,
}: RadioCategoryTileProps) => (
  <button
    type="button"
    onClick={() => onSelect(label, term)}
    className="flex h-[54px] items-center gap-[10px] rounded-control bg-panel pl-3 pr-[10px] text-left transition-colors hover:bg-hover"
  >
    <Icon size={20} style={{ color }} className="shrink-0" />
    <span className="truncate text-xs font-semibold text-fg">{label}</span>
  </button>
);

export default RadioRow;
