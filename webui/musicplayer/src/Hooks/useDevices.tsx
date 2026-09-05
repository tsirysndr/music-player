import { useQueryClient } from "@tanstack/react-query";
import { useAtomValue } from "jotai";
import {
  castDevicesAtom,
  currentCastDeviceAtom,
  currentDeviceAtom,
  devicesAtom,
} from "../State";
import {
  ConnectToCastDeviceMutationVariables,
  ConnectToDeviceMutationVariables,
  useConnectedCastDeviceQuery,
  useConnectedDeviceQuery,
  useConnectToCastDeviceMutation,
  useConnectToDeviceMutation,
  useDisconnectFromCastDeviceMutation,
  useDisconnectFromDeviceMutation,
} from "./GraphQL";

export const useDevices = () => {
  const devices = useAtomValue(devicesAtom);
  const castDevices = useAtomValue(castDevicesAtom);
  const currentDevice = useAtomValue(currentDeviceAtom);
  const currentCastDevice = useAtomValue(currentCastDeviceAtom);
  const queryClient = useQueryClient();

  const invalidateConnectedDevices = () => {
    queryClient.invalidateQueries({
      queryKey: useConnectedDeviceQuery.getKey(),
    });
    queryClient.invalidateQueries({
      queryKey: useConnectedCastDeviceQuery.getKey(),
    });
  };

  const connectToDeviceMutation = useConnectToDeviceMutation({
    onSuccess: invalidateConnectedDevices,
  });
  const disconnectFromDeviceMutation = useDisconnectFromDeviceMutation({
    onSuccess: invalidateConnectedDevices,
  });
  const connectToCastDeviceMutation = useConnectToCastDeviceMutation({
    onSuccess: invalidateConnectedDevices,
  });
  const disconnectFromCastDeviceMutation = useDisconnectFromCastDeviceMutation({
    onSuccess: invalidateConnectedDevices,
  });

  return {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice: (variables: ConnectToDeviceMutationVariables) =>
      connectToDeviceMutation.mutateAsync(variables),
    disconnectFromDevice: () => disconnectFromDeviceMutation.mutateAsync({}),
    connectToCastDevice: (variables: ConnectToCastDeviceMutationVariables) =>
      connectToCastDeviceMutation.mutateAsync(variables),
    disconnectFromCastDevice: () =>
      disconnectFromCastDeviceMutation.mutateAsync({}),
  };
};
