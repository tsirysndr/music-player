import { Link } from "react-router-dom";
import IconButton from "./IconButton";
import { Icons } from "./icons";

export type PlaylistRowItem = {
  id: string;
  name: string;
  description?: string | null;
  trackCount?: number;
};

export type PlaylistRowProps = {
  playlist: PlaylistRowItem;
  onPlay?: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
};

const trackLabel = (count?: number) =>
  count === 1 ? "1 track" : `${count ?? 0} tracks`;

/**
 * The desktop's `PlaylistRow`: name over a description-and-count line, with
 * edit, delete and play at the right.
 */
const PlaylistRow = ({
  playlist,
  onPlay,
  onEdit,
  onDelete,
}: PlaylistRowProps) => {
  const count = trackLabel(playlist.trackCount);
  return (
    <div className="group flex h-14 items-center gap-3 rounded-control bg-panel pl-[14px] pr-[10px] hover:bg-hover">
      <Icons.playlist size={18} className="shrink-0 text-accent" />
      <Link
        to={`/playlists/${playlist.id}`}
        className="flex min-w-0 flex-1 flex-col justify-center gap-[2px]"
      >
        <span className="truncate text-[13px] font-semibold text-fg">
          {playlist.name}
        </span>
        <span className="truncate text-[11px] text-dim">
          {playlist.description ? `${playlist.description} · ${count}` : count}
        </span>
      </Link>
      {onEdit && (
        <IconButton
          icon={Icons.pencil}
          iconSize={14}
          aria-label={`Edit ${playlist.name}`}
          onClick={onEdit}
          className="opacity-35 transition-opacity group-hover:opacity-100"
        />
      )}
      {onDelete && (
        <IconButton
          icon={Icons.trash}
          iconSize={14}
          aria-label={`Delete ${playlist.name}`}
          onClick={onDelete}
          className="opacity-35 transition-opacity group-hover:opacity-100"
        />
      )}
      {onPlay && (
        <IconButton
          icon={Icons.play}
          iconSize={15}
          accented
          aria-label={`Play ${playlist.name}`}
          onClick={onPlay}
        />
      )}
    </div>
  );
};

export default PlaylistRow;
