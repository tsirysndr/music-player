import { useQueryClient } from "@tanstack/react-query";
import {
  AddTrackToPlaylistMutationVariables,
  CreateFolderMutationVariables,
  CreatePlaylistMutationVariables,
  DeleteFolderMutationVariables,
  DeletePlaylistMutationVariables,
  GetFolderQueryVariables,
  GetPlaylistQueryVariables,
  MovePlaylistsToFolderMutationVariables,
  MovePlaylistToFolderMutationVariables,
  RenameFolderMutationVariables,
  RenamePlaylistMutationVariables,
  useAddTrackToPlaylistMutation,
  useCreateFolderMutation,
  useCreatePlaylistMutation,
  useDeleteFolderMutation,
  useDeletePlaylistMutation,
  useGetFolderQuery,
  useGetFoldersQuery,
  useGetMainPlaylistsQuery,
  useGetPlaylistQuery,
  useGetPlaylistsQuery,
  useGetRecentPlaylistsQuery,
  useMovePlaylistsToFolderMutation,
  useMovePlaylistToFolderMutation,
  useRemoveTrackFromPlaylistMutation,
  useRenameFolderMutation,
  useRenamePlaylistMutation,
  type RemoveTrackFromPlaylistMutationVariables,
} from "./GraphQL";

export const usePlaylist = () => {
  const queryClient = useQueryClient();

  const { data: playlistsData } = useGetPlaylistsQuery(undefined, {
    refetchInterval: 5000,
  });
  const { data: recentPlaylistsData } = useGetRecentPlaylistsQuery(undefined, {
    refetchInterval: 5000,
  });
  const { data: mainPlaylistsData } = useGetMainPlaylistsQuery(undefined, {
    refetchInterval: 5000,
  });
  const { data: foldersData } = useGetFoldersQuery(undefined, {
    refetchInterval: 5000,
  });

  const invalidatePlaylists = () => {
    queryClient.invalidateQueries({ queryKey: useGetPlaylistsQuery.getKey() });
    queryClient.invalidateQueries({
      queryKey: useGetRecentPlaylistsQuery.getKey(),
    });
    queryClient.invalidateQueries({
      queryKey: useGetMainPlaylistsQuery.getKey(),
    });
    queryClient.invalidateQueries({ queryKey: useGetFoldersQuery.getKey() });
    // invalidate every GetPlaylist / GetFolder query, whatever their variables
    queryClient.invalidateQueries({ queryKey: ["GetPlaylist"] });
    queryClient.invalidateQueries({ queryKey: ["GetFolder"] });
  };

  const createFolderMutation = useCreateFolderMutation({
    onSuccess: invalidatePlaylists,
  });
  const createPlaylistMutation = useCreatePlaylistMutation({
    onSuccess: invalidatePlaylists,
  });
  const addTrackToPlaylistMutation = useAddTrackToPlaylistMutation({
    onSuccess: invalidatePlaylists,
  });
  const movePlaylistToFolderMutation = useMovePlaylistToFolderMutation({
    onSuccess: invalidatePlaylists,
  });
  const movePlaylistsToFolderMutation = useMovePlaylistsToFolderMutation({
    onSuccess: invalidatePlaylists,
  });
  const deleteFolderMutation = useDeleteFolderMutation({
    onSuccess: invalidatePlaylists,
  });
  const deletePlaylistMutation = useDeletePlaylistMutation({
    onSuccess: invalidatePlaylists,
  });
  const renamePlaylistMutation = useRenamePlaylistMutation({
    onSuccess: invalidatePlaylists,
  });
  const renameFolderMutation = useRenameFolderMutation({
    onSuccess: invalidatePlaylists,
  });
  const removeTrackFromPlaylistMutation = useRemoveTrackFromPlaylistMutation({
    onSuccess: invalidatePlaylists,
  });

  const playlists = playlistsData?.playlists || [];
  const folders = foldersData?.folders || [];
  const recentPlaylists = recentPlaylistsData?.recentPlaylists || [];
  const mainPlaylists = mainPlaylistsData?.mainPlaylists || [];

  return {
    playlists,
    folders,
    recentPlaylists,
    mainPlaylists,
    getPlaylist: (variables: GetPlaylistQueryVariables) =>
      queryClient.fetchQuery({
        queryKey: useGetPlaylistQuery.getKey(variables),
        queryFn: useGetPlaylistQuery.fetcher(variables),
      }),
    getFolder: (variables: GetFolderQueryVariables) =>
      queryClient.fetchQuery({
        queryKey: useGetFolderQuery.getKey(variables),
        queryFn: useGetFolderQuery.fetcher(variables),
      }),
    createFolder: (variables: CreateFolderMutationVariables) =>
      createFolderMutation.mutateAsync(variables),
    createPlaylist: (variables: CreatePlaylistMutationVariables) =>
      createPlaylistMutation.mutateAsync(variables),
    addTrackToPlaylist: (variables: AddTrackToPlaylistMutationVariables) =>
      addTrackToPlaylistMutation.mutateAsync(variables),
    movePlaylistToFolder: (variables: MovePlaylistToFolderMutationVariables) =>
      movePlaylistToFolderMutation.mutateAsync(variables),
    movePlaylistsToFolder: (
      variables: MovePlaylistsToFolderMutationVariables
    ) => movePlaylistsToFolderMutation.mutateAsync(variables),
    deleteFolder: (variables: DeleteFolderMutationVariables) =>
      deleteFolderMutation.mutateAsync(variables),
    deletePlaylist: (variables: DeletePlaylistMutationVariables) =>
      deletePlaylistMutation.mutateAsync(variables),
    renamePlaylist: (variables: RenamePlaylistMutationVariables) =>
      renamePlaylistMutation.mutateAsync(variables),
    renameFolder: (variables: RenameFolderMutationVariables) =>
      renameFolderMutation.mutateAsync(variables),
    removeTrackFromPlaylist: (
      variables: RemoveTrackFromPlaylistMutationVariables
    ) => removeTrackFromPlaylistMutation.mutateAsync(variables),
  };
};
