import { FC, useMemo, useState } from "react";
import {
  Button,
  Dialog,
  EmptyState,
  FilterBox,
  Icons,
  cn,
} from "../UI";

export type MovePlaylistsModalProps = {
  isOpen: boolean;
  onClose: () => void;
  onMovePlaylists: (playlistIds: string[], folderId: string) => void;
  playlists: { id: string; name: string }[];
  folderId: string;
};

/** Pick playlists to move into this folder. */
const MovePlaylistsModal: FC<MovePlaylistsModalProps> = ({
  isOpen,
  onClose,
  onMovePlaylists,
  playlists,
  folderId,
}) => {
  const [selected, setSelected] = useState<string[]>([]);
  const [filter, setFilter] = useState("");

  const rows = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    return needle
      ? playlists.filter((playlist) =>
          playlist.name.toLowerCase().includes(needle)
        )
      : playlists;
  }, [playlists, filter]);

  const close = () => {
    onClose();
    setSelected([]);
    setFilter("");
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={close}
      title="Move playlists"
      icon={Icons.folder}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button
            disabled={selected.length === 0}
            onClick={() => {
              onMovePlaylists(selected, folderId);
              close();
            }}
          >
            {selected.length === 1
              ? "Move 1 playlist"
              : `Move ${selected.length} playlists`}
          </Button>
        </>
      }
    >
      <FilterBox
        value={filter}
        placeholder="Filter playlists…"
        className="mb-2"
        onChange={setFilter}
      />
      {rows.length === 0 ? (
        <EmptyState icon={Icons.playlist} title="No playlists to move" />
      ) : (
        <ul className="flex flex-col gap-1 pb-1">
          {rows.map((playlist) => {
            const checked = selected.includes(playlist.id);
            return (
              <li key={playlist.id}>
                <button
                  type="button"
                  aria-pressed={checked}
                  onClick={() =>
                    setSelected((current) =>
                      checked
                        ? current.filter((id) => id !== playlist.id)
                        : [...current, playlist.id]
                    )
                  }
                  className={cn(
                    "flex h-11 w-full items-center gap-3 rounded-control px-3 text-left",
                    checked ? "bg-selected" : "hover:bg-hover"
                  )}
                >
                  <span
                    className={cn(
                      "grid size-[18px] shrink-0 place-items-center rounded border",
                      checked
                        ? "border-accent bg-accent text-on-accent"
                        : "border-line"
                    )}
                  >
                    {checked && <Icons.check size={12} stroke={3} />}
                  </span>
                  <span className="truncate text-[13px] text-fg">
                    {playlist.name}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </Dialog>
  );
};

export default MovePlaylistsModal;
