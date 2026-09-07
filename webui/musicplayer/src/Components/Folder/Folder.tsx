import { FC, useMemo, useState } from "react";
import { AppShell } from "../Layout";
import {
  Button,
  EmptyState,
  Icons,
  PageToolbar,
  PlaylistRow,
  type PlaylistRowItem,
} from "../UI";
import MovePlaylistsModal from "./MovePlaylistsModal";

export type FolderProps = {
  folder?: { id: string; name: string; playlists: PlaylistRowItem[] };
  loading?: boolean;
  /** Every playlist, for the "move into this folder" picker. */
  allPlaylists: { id: string; name: string }[];
  onBack: () => void;
  onMovePlaylists: (playlistIds: string[], folderId: string) => void;
  onPlayPlaylist: (id: string) => void;
};

/** A folder of playlists. */
const Folder: FC<FolderProps> = ({
  folder,
  loading,
  allPlaylists,
  onBack,
  onMovePlaylists,
  onPlayPlaylist,
}) => {
  const [filter, setFilter] = useState("");
  const [moving, setMoving] = useState(false);

  const rows = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    const playlists = folder?.playlists ?? [];
    return needle
      ? playlists.filter((playlist) =>
          playlist.name.toLowerCase().includes(needle)
        )
      : playlists;
  }, [folder, filter]);

  return (
    <AppShell title={folder?.name ?? "Folder"} onBack={onBack}>
      <PageToolbar
        filter={filter}
        filterPlaceholder="Filter playlists…"
        onFilter={
          (folder?.playlists.length ?? 0) > 0 ? setFilter : undefined
        }
      >
        <Button
          variant="outline"
          icon={Icons.circlePlus}
          onClick={() => setMoving(true)}
        >
          Move playlists here
        </Button>
      </PageToolbar>

      {loading ? null : rows.length === 0 ? (
        <EmptyState
          icon={Icons.folder}
          title={
            filter
              ? `Nothing matches “${filter}”`
              : "Start moving playlists to your folder"
          }
          action={
            filter ? undefined : (
              <Button icon={Icons.circlePlus} onClick={() => setMoving(true)}>
                Move playlists
              </Button>
            )
          }
        />
      ) : (
        <div className="flex flex-col gap-[6px]">
          {rows.map((playlist) => (
            <PlaylistRow
              key={playlist.id}
              playlist={playlist}
              onPlay={() => onPlayPlaylist(playlist.id)}
            />
          ))}
        </div>
      )}

      <MovePlaylistsModal
        isOpen={moving}
        onClose={() => setMoving(false)}
        onMovePlaylists={onMovePlaylists}
        playlists={allPlaylists}
        folderId={folder?.id ?? ""}
      />
    </AppShell>
  );
};

export default Folder;
