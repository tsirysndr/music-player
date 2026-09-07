import { Link } from "react-router-dom";
import Artwork from "./Artwork";
import cn from "./cn";
import ContextMenu, {
  ContextMenuItem,
  ContextMenuSeparator,
} from "./ContextMenu";
import { Icons } from "./icons";

export type AlbumCardItem = {
  id: string;
  title: string;
  artist: string;
  year?: number | string | null;
  cover?: string | null;
};

export type AlbumCardProps = {
  album: AlbumCardItem;
  /** Where clicking the card goes. Falls back to the album detail route. */
  to?: string;
  onPlay?: () => void;
  onShufflePlay?: () => void;
  onPlayNext?: () => void;
  onAddToQueue?: () => void;
  onLike?: () => void;
  className?: string;
};

/**
 * The desktop's `AlbumCard`: a panel that raises on hover and reveals a play
 * disc over the bottom-right of the art and a "…" menu over the bottom-left.
 *
 * Both overlay buttons sit outside the card's `<Link>` rather than inside it —
 * nesting a button in an anchor is invalid, and clicking play would navigate.
 */
const AlbumCard = ({
  album,
  to,
  onPlay,
  onShufflePlay,
  onPlayNext,
  onAddToQueue,
  onLike,
  className,
}: AlbumCardProps) => (
  <div
    className={cn(
      "group relative rounded-skin bg-panel p-3 transition-colors duration-150 hover:bg-raised",
      className
    )}
  >
    <Link to={to ?? `/albums/${album.id}`} className="block">
      <Artwork
        src={album.cover}
        alt={album.title}
        fallbackIcon={Icons.disc}
        className="aspect-square w-full"
      />
      <div className="mt-2 truncate text-[13px] font-semibold text-fg">
        {album.title}
      </div>
      <div className="mt-[2px] truncate text-[11px] text-dim">
        {album.year ? `${album.artist} · ${album.year}` : album.artist}
      </div>
    </Link>

    {onPlay && (
      <button
        type="button"
        aria-label={`Play ${album.title}`}
        onClick={onPlay}
        className={cn(
          "absolute right-5 grid size-[38px] place-items-center rounded-full",
          "bg-accent text-on-accent shadow-[0_2px_8px_#00000080]",
          "opacity-0 transition-opacity duration-150 hover:bg-accent-hover",
          "group-hover:opacity-100 focus-visible:opacity-100"
        )}
        // The art is square, so its bottom edge is 12px (padding) below
        // `top = card width - padding`; 20px in from the corner matches Slint.
        style={{ top: "calc(100% - 66px - 20px)" }}
      >
        <Icons.play size={16} className="translate-x-[1px]" />
      </button>
    )}

    {(onShufflePlay || onPlayNext || onAddToQueue || onLike) && (
      <div
        className="absolute left-5 opacity-0 transition-opacity duration-150 group-hover:opacity-100 focus-within:opacity-100"
        style={{ top: "calc(100% - 66px - 16px)" }}
      >
        <ContextMenu
          width={190}
          placement="bottom start"
          trigger={
            <button
              type="button"
              aria-label={`More actions for ${album.title}`}
              className="grid size-[30px] place-items-center rounded-full bg-black/70 text-white"
            >
              <Icons.ellipsis size={15} />
            </button>
          }
        >
          {onPlay && (
            <ContextMenuItem icon={Icons.play} label="Play" onClick={onPlay} />
          )}
          {onShufflePlay && (
            <ContextMenuItem
              icon={Icons.shuffle}
              label="Shuffle play"
              onClick={onShufflePlay}
            />
          )}
          {onPlayNext && (
            <ContextMenuItem
              icon={Icons.next}
              label="Play next"
              onClick={onPlayNext}
            />
          )}
          {onAddToQueue && (
            <ContextMenuItem
              icon={Icons.listMusic}
              label="Add to queue"
              onClick={onAddToQueue}
            />
          )}
          {onLike && (
            <>
              <ContextMenuSeparator />
              <ContextMenuItem
                icon={Icons.heart}
                label="Like album"
                onClick={onLike}
              />
            </>
          )}
        </ContextMenu>
      </div>
    )}
  </div>
);

export default AlbumCard;
