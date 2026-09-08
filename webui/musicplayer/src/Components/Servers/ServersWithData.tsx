import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  useAddServerMutation,
  useConnectToServerMutation,
  useDeleteServerMutation,
  useDisconnectFromServerMutation,
  useGetSavedServersQuery,
  useGetSourceKindsQuery,
} from "../../Hooks/GraphQL";
import AddServerForm, { type AddServerValues } from "./AddServerForm";
import Servers, { type ServerItem } from "./Servers";

/**
 * Everything the library screens read changes when the connected server does,
 * so a connect or disconnect invalidates the lot.
 *
 * Deliberately *not* a page reload, which is what the device-connect path used
 * to do: audio is produced by the daemon, so a reload would not stop playback,
 * but it tears down the subscriptions and flashes the whole app for no reason.
 */
const LIBRARY_QUERIES = [
  "GetTracks",
  "GetAlbums",
  "GetArtists",
  "GetAlbum",
  "GetArtist",
  "GetPlaylists",
  "GetPlaylist",
  "GetLikedTracks",
  "Search",
];

const ServersWithData = () => {
  const queryClient = useQueryClient();
  const [addOpen, setAddOpen] = useState(false);
  const [busyId, setBusyId] = useState<string>();
  const [error, setError] = useState<string>();
  const [formError, setFormError] = useState<string>();

  const { data, isLoading, refetch } = useGetSavedServersQuery();
  const { data: kindData } = useGetSourceKindsQuery();

  const refreshLibrary = async () => {
    await refetch();
    await Promise.all(
      LIBRARY_QUERIES.map((name) =>
        queryClient.invalidateQueries({
          predicate: (query) => query.queryKey[0] === name,
        })
      )
    );
  };

  const addServer = useAddServerMutation();
  const connect = useConnectToServerMutation();
  const disconnect = useDisconnectFromServerMutation();
  const remove = useDeleteServerMutation();

  const servers: ServerItem[] = (data?.savedServers ?? []).map((server) => ({
    id: server.id,
    kind: server.kind,
    name: server.name,
    url: server.url,
    username: server.username,
    connected: server.connected,
  }));

  const connectTo = async (id: string) => {
    setBusyId(id);
    setError(undefined);
    try {
      await connect.mutateAsync({ id });
      await refreshLibrary();
    } catch (cause) {
      setError(message(cause));
      throw cause;
    } finally {
      setBusyId(undefined);
    }
  };

  return (
    <>
      <Servers
        servers={servers}
        kinds={kindData?.sourceKinds ?? []}
        loading={isLoading}
        error={error}
        busyId={busyId}
        onAdd={() => {
          setFormError(undefined);
          setAddOpen(true);
        }}
        onConnect={(server) => {
          connectTo(server.id).catch(() => {});
        }}
        onDisconnect={async () => {
          setError(undefined);
          try {
            await disconnect.mutateAsync({});
            await refreshLibrary();
          } catch (cause) {
            setError(message(cause));
          }
        }}
        onDelete={async (server) => {
          setError(undefined);
          try {
            await remove.mutateAsync({ id: server.id });
            // Deleting the connected one drops back to the local library, so
            // the screens have to be re-read either way.
            await refreshLibrary();
          } catch (cause) {
            setError(message(cause));
          }
        }}
      />
      <AddServerForm
        isOpen={addOpen}
        kinds={kindData?.sourceKinds ?? []}
        error={formError}
        submitting={addServer.isPending || connect.isPending}
        onClose={() => setAddOpen(false)}
        onSubmit={async (values: AddServerValues) => {
          setFormError(undefined);
          try {
            // Saved and connected in one go: adding a server is only ever a
            // step towards using it.
            const added = await addServer.mutateAsync({ input: values });
            await connectTo(added.addServer.id);
            setAddOpen(false);
          } catch (cause) {
            setFormError(message(cause));
          }
        }}
      />
    </>
  );
};

/** The daemon's own words, which say what actually went wrong. */
const message = (cause: unknown) =>
  cause instanceof Error ? cause.message : "Something went wrong";

export default ServersWithData;
