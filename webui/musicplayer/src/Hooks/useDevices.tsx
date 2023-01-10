import _ from "lodash";
import { useEffect, useState } from "react";
import { Device } from "../Types/Device";
import {
  useConnectedDeviceQuery,
  useConnectToDeviceMutation,
  useDisconnectFromDeviceMutation,
  useListDevicesQuery,
  useOnNewDeviceSubscription,
} from "./GraphQL";

export const useDevices = () => {
  const [devices, setDevices] = useState<Device[]>([]);
  const { data } = useOnNewDeviceSubscription();
  const { data: listDevicesData } = useListDevicesQuery();
  const { data: connectedDeviceData } = useConnectedDeviceQuery();
  const [connectToDevice] = useConnectToDeviceMutation();
  const [disconnectFromDevice] = useDisconnectFromDeviceMutation();

  useEffect(() => {
    if (
      data?.onNewDevice &&
      (data.onNewDevice.app === "music-player" ||
        data?.onNewDevice.app === "xbmc")
    ) {
      setDevices(
        _.uniqBy(
          [
            ...devices,
            {
              id: data.onNewDevice.id,
              type: data.onNewDevice.app,
              name: data.onNewDevice.name,
              isConnected: data.onNewDevice.isConnected,
            },
          ],
          "id"
        )
      );
    }
    if (listDevicesData?.listDevices && listDevicesData.listDevices.length) {
      setDevices(
        _.uniqBy(
          listDevicesData.listDevices.map((x) => ({
            id: x.id,
            type: x.app,
            name: x.name,
            isConnected: x.isConnected,
          })),
          "id"
        )
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, listDevicesData]);

  const currentDevice: Device | undefined = connectedDeviceData
    ? {
        id: connectedDeviceData.connectedDevice.id,
        type: connectedDeviceData.connectedDevice.app,
        name: connectedDeviceData.connectedDevice.name,
        isConnected: connectedDeviceData.connectedDevice.isConnected,
      }
    : undefined;

  return { devices, currentDevice, connectToDevice, disconnectFromDevice };
};
