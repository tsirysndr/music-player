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
  useGetPlaylistLazyQuery,
  useGetPlaylistsLazyQuery,
  useGetPlaylistsQuery,
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
  const [deleteFolder] = useDeleteFolderMutation();
  const [deletePlaylist] = useDeletePlaylistMutation();
  const [renamePlaylist] = useRenamePlaylistMutation();
  const [renameFolder] = useRenameFolderMutation();

  const playlists = playlistsData?.playlists || [];
  const folders = foldersData?.folders || [];

  useEffect(() => {
    startPollingPlaylists!(1000);
    startPollingFolders(1000);
    return () => {
      stopPollingPlaylists();
      stopPollingFolders();
    };
  }, [
    startPollingFolders,
    startPollingPlaylists,
    stopPollingFolders,
    stopPollingPlaylists,
  ]);

  return {
    playlists,
    folders,
    getPlaylist,
    getPlaylists,
    getFolder,
    getFolders,
    createFolder,
    createPlaylist,
    addTrackToPlaylist,
    movePlaylistToFolder,
    deleteFolder,
    deletePlaylist,
    renamePlaylist,
    renameFolder,
  };
};
