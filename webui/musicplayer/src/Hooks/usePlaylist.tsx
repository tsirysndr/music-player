import { useEffect } from "react";
import {
  useAddTrackToPlaylistMutation,
  useCreateFolderMutation,
  useCreatePlaylistMutation,
  useDeleteFolderMutation,
  useDeletePlaylistMutation,
  useGetFolderLazyQuery,
  useGetFoldersLazyQuery,
  useGetFoldersQuery,
  useGetMainPlaylistsQuery,
  useGetPlaylistLazyQuery,
  useGetPlaylistsLazyQuery,
  useGetPlaylistsQuery,
  useGetRecentPlaylistsQuery,
  useMovePlaylistsToFolderMutation,
  useMovePlaylistToFolderMutation,
  useRenameFolderMutation,
  useRenamePlaylistMutation,
} from "./GraphQL";

export const usePlaylist = () => {
  const [getPlaylist] = useGetPlaylistLazyQuery();
  const [getPlaylists] = useGetPlaylistsLazyQuery();
  const {
    data: playlistsData,
    startPolling: startPollingPlaylists,
    stopPolling: stopPollingPlaylists,
  } = useGetPlaylistsQuery({
    pollInterval: 1000,
  });
  const {
    data: recentPlaylistsData,
    startPolling: startPollingRecentPlaylists,
    stopPolling: stopPollingRecentPlaylists,
  } = useGetRecentPlaylistsQuery({
    pollInterval: 1000,
  });
  const {
    data: mainPlaylistsData,
    startPolling: startPollingMainPlaylists,
    stopPolling: stopPollingMainPlaylists,
  } = useGetMainPlaylistsQuery({
    pollInterval: 1000,
  });
  const {
    data: foldersData,
    startPolling: startPollingFolders,
    stopPolling: stopPollingFolders,
  } = useGetFoldersQuery({
    pollInterval: 1000,
  });
  const [getFolder] = useGetFolderLazyQuery();
  const [getFolders] = useGetFoldersLazyQuery();
  const [createFolder] = useCreateFolderMutation();
  const [createPlaylist] = useCreatePlaylistMutation();
  const [addTrackToPlaylist] = useAddTrackToPlaylistMutation();
  const [movePlaylistToFolder] = useMovePlaylistToFolderMutation();
  const [movePlaylistsToFolder] = useMovePlaylistsToFolderMutation();
  const [deleteFolder] = useDeleteFolderMutation();
  const [deletePlaylist] = useDeletePlaylistMutation();
  const [renamePlaylist] = useRenamePlaylistMutation();
  const [renameFolder] = useRenameFolderMutation();

  const playlists = playlistsData?.playlists || [];
  const folders = foldersData?.folders || [];
  const recentPlaylists = recentPlaylistsData?.recentPlaylists || [];
  const mainPlaylists = mainPlaylistsData?.mainPlaylists || [];

  useEffect(() => {
    startPollingPlaylists!(1000);
    startPollingFolders(1000);
    startPollingMainPlaylists(1000);
    startPollingRecentPlaylists(1000);
    return () => {
      stopPollingPlaylists();
      stopPollingFolders();
      stopPollingRecentPlaylists();
      stopPollingRecentPlaylists();
    };
  }, [
    startPollingFolders,
    startPollingPlaylists,
    startPollingRecentPlaylists,
    startPollingMainPlaylists,
    stopPollingFolders,
    stopPollingPlaylists,
    stopPollingRecentPlaylists,
    stopPollingMainPlaylists,
  ]);

  return {
    playlists,
    folders,
    recentPlaylists,
    mainPlaylists,
    getPlaylist,
    getPlaylists,
    getFolder,
    getFolders,
    createFolder,
    createPlaylist,
    addTrackToPlaylist,
    movePlaylistToFolder,
    movePlaylistsToFolder,
    deleteFolder,
    deletePlaylist,
    renamePlaylist,
    renameFolder,
  };
};
