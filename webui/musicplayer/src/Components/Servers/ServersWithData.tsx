import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  useListCastDevicesQuery,
  useListDevicesQuery,
} from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import Servers, { type ServerItem } from "./Servers";

/** The atoms carry `type`; the page wants the daemon's word for it, `kind`. */
const toItem = (
  device: { id: string; name: string; type: string },
  cast: boolean
): ServerItem => ({
  id: device.id,
  name: device.name,
  kind: device.type,
  cast,
});

const ServersWithData = () => {
  const {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    connectToCastDevice,
    disconnectFromDevice,
    disconnectFromCastDevice,
  } = useDevices();
  const queryClient = useQueryClient();
  const [busyId, setBusyId] = useState<string>();
  const [error, setError] = useState<string>();

  // The lists themselves live in atoms fed by `useDevicesSync`; these two are
  // only here for the loading state and so the refresh button has something
  // to invalidate.
  const { isLoading: devicesLoading } = useListDevicesQuery();
  const { isLoading: castLoading } = useListCastDevicesQuery();

  const connect = async (server: ServerItem) => {
    setBusyId(server.id);
    setError(undefined);
    try {
      if (server.cast) await connectToCastDevice({ id: server.id });
      else await connectToDevice({ id: server.id });
    } catch (cause) {
      setError(
        `Could not connect to ${server.name}: ${
          cause instanceof Error ? cause.message : "unknown error"
        }`
      );
    } finally {
      setBusyId(undefined);
    }
  };

  return (
    <Servers
      servers={devices.map((device) => toItem(device, false))}
      castDevices={castDevices.map((device) => toItem(device, true))}
      connectedId={currentCastDevice?.id ?? currentDevice?.id}
      loading={devicesLoading || castLoading}
      error={error}
      busyId={busyId}
      onConnect={connect}
      onDisconnect={() => {
        if (currentCastDevice) disconnectFromCastDevice();
        if (currentDevice) disconnectFromDevice();
      }}
      onRefresh={() => {
        setError(undefined);
        queryClient.invalidateQueries({
          queryKey: useListDevicesQuery.getKey(),
        });
        queryClient.invalidateQueries({
          queryKey: useListCastDevicesQuery.getKey(),
        });
      }}
    />
  );
};

export default ServersWithData;
