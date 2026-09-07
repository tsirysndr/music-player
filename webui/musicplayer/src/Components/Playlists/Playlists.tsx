import { FC, useState } from "react";
import { Link } from "react-router-dom";
import { AppShell } from "../Layout";
import {
  Button,
  EmptyState,
  Icons,
  PageToolbar,
  PlaylistRow,
  type PlaylistRowItem,
} from "../UI";
import DeleteConfirmationModal from "./DeleteConfirmationModal";
import EditPlaylistModal from "./EditPlaylistModal";
import NewFolderModal from "./NewFolderModal";
import NewPlaylistModal, { type SmartPlaylistRule } from "./NewPlaylistModal";

export type FolderItem = { id: string; name: string };

export type PlaylistsProps = {
  playlists: PlaylistRowItem[];
  folders: FolderItem[];
  onCreatePlaylist: (
    name: string,
    description?: string,
    smart?: SmartPlaylistRule
  ) => void;
  onCreateFolder: (name: string) => void;
  onEditPlaylist: (id: string, name: string, description?: string) => void;
  onDeletePlaylist: (id: string) => void;
  onPlayPlaylist: (id: string) => void;
};

/**
 * The desktop's playlists tab: a "New playlist" pill over the list, with the
 * folders above it.
 */
const Playlists: FC<PlaylistsProps> = ({
  playlists,
  folders,
  onCreatePlaylist,
  onCreateFolder,
  onEditPlaylist,
  onDeletePlaylist,
  onPlayPlaylist,
}) => {
  const [newPlaylist, setNewPlaylist] = useState(false);
  const [newFolder, setNewFolder] = useState(false);
  const [editing, setEditing] = useState<PlaylistRowItem | undefined>();
  const [deleting, setDeleting] = useState<PlaylistRowItem | undefined>();

  return (
    <AppShell>
      {/* No filter here: playlists are reachable from the command palette
          (`/` or ⌘K), which searches them alongside everything else. */}
      <PageToolbar>
        <Button icon={Icons.circlePlus} onClick={() => setNewPlaylist(true)}>
          New playlist
        </Button>
        <Button
          variant="outline"
          icon={Icons.folder}
          onClick={() => setNewFolder(true)}
        >
          New folder
        </Button>
      </PageToolbar>

      {folders.length > 0 && (
        <div className="mb-6 flex flex-wrap gap-2">
          {folders.map((folder) => (
            <Link
              key={folder.id}
              to={`/folders/${folder.id}`}
              className="flex h-9 items-center gap-2 rounded-control bg-panel px-3 text-[13px] text-fg hover:bg-hover"
            >
              <Icons.folder size={15} className="text-accent" />
              {folder.name}
            </Link>
          ))}
        </div>
      )}

      {playlists.length === 0 ? (
        <EmptyState
          icon={Icons.playlist}
          title="No playlists yet — create one!"
          action={
            <Button icon={Icons.circlePlus} onClick={() => setNewPlaylist(true)}>
              New playlist
            </Button>
          }
        />
      ) : (
        <div className="flex flex-col gap-[6px]">
          {playlists.map((playlist) => (
            <PlaylistRow
              key={playlist.id}
              playlist={playlist}
              onPlay={() => onPlayPlaylist(playlist.id)}
              onEdit={() => setEditing(playlist)}
              onDelete={() => setDeleting(playlist)}
            />
          ))}
        </div>
      )}

      <NewPlaylistModal
        isOpen={newPlaylist}
        onClose={() => setNewPlaylist(false)}
        onCreatePlaylist={onCreatePlaylist}
      />
      <NewFolderModal
        isOpen={newFolder}
        onClose={() => setNewFolder(false)}
        onCreateFolder={onCreateFolder}
      />
      <EditPlaylistModal
        isOpen={!!editing}
        playlist={editing}
        onClose={() => setEditing(undefined)}
        onEditPlaylist={onEditPlaylist}
      />
      <DeleteConfirmationModal
        isOpen={!!deleting}
        title="Delete playlist"
        message={`“${deleting?.name}” will be removed. The tracks stay in your library.`}
        onClose={() => setDeleting(undefined)}
        onDelete={() => deleting && onDeletePlaylist(deleting.id)}
      />
    </AppShell>
  );
};

export default Playlists;
