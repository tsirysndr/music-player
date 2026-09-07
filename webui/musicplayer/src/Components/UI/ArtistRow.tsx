import { Link } from "react-router-dom";
import Artwork from "./Artwork";
import IconButton from "./IconButton";
import { Icons } from "./icons";

export type ArtistRowItem = {
  id: string;
  name: string;
  picture?: string | null;
};

export type ArtistRowProps = {
  artist: ArtistRowItem;
  to?: string;
  onPlay?: () => void;
};

/** The desktop's `ArtistRow`: round art, name, and a play button on hover. */
const ArtistRow = ({ artist, to, onPlay }: ArtistRowProps) => (
  <div className="group flex h-14 items-center gap-[14px] rounded-control px-3 hover:bg-hover">
    <Link
      to={to ?? `/artists/${artist.id}`}
      className="flex min-w-0 flex-1 items-center gap-[14px]"
    >
      <Artwork
        src={artist.picture}
        alt={artist.name}
        fallbackIcon={Icons.artist}
        iconSize={18}
        rounded="full"
        className="size-10"
      />
      <span className="truncate text-[13px] text-fg">{artist.name}</span>
    </Link>
    {onPlay && (
      <IconButton
        icon={Icons.play}
        iconSize={14}
        aria-label={`Play ${artist.name}`}
        onClick={onPlay}
        className="opacity-35 transition-opacity group-hover:opacity-100"
      />
    )}
  </div>
);

export default ArtistRow;
