import { useSnackbar } from "baseui/snackbar";
import _ from "lodash";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Device } from "../Types/Device";
import {
  useConnectedCastDeviceQuery,
  useConnectedDeviceQuery,
  useConnectToCastDeviceMutation,
  useConnectToDeviceMutation,
  useDisconnectFromCastDeviceMutation,
  useDisconnectFromDeviceMutation,
  useListCastDevicesQuery,
  useListDevicesQuery,
  useOnDeviceConnectedSubscription,
  useOnDeviceDisconnectedSubscription,
  useOnNewDeviceSubscription,
} from "./GraphQL";

export const useDevices = () => {
  const navigate = useNavigate();
  const [currentDevice, setCurrentDevice] =
    useState<Device | undefined>(undefined);
  const [currentCastDevice, setCurrentCastDevice] =
    useState<Device | undefined>(undefined);
  const [devices, setDevices] = useState<Device[]>([]);
  const [castDevices, setCastDevices] = useState<Device[]>([]);
  const { data } = useOnNewDeviceSubscription();
  const { data: listDevicesData } = useListDevicesQuery();
  const { data: listCastDevicesData } = useListCastDevicesQuery();
  const { data: connectedDeviceData, refetch } = useConnectedDeviceQuery();
  const { data: connectedCastDeviceData } = useConnectedCastDeviceQuery();
  const [connectToDevice] = useConnectToDeviceMutation();
  const [disconnectFromDevice] = useDisconnectFromDeviceMutation();
  const [connectToCastDevice] = useConnectToCastDeviceMutation();
  const [disconnectFromCastDevice] = useDisconnectFromCastDeviceMutation();
  const { data: deviceConnectedData } = useOnDeviceConnectedSubscription();
  const { data: deviceDisconnectedData } =
    useOnDeviceDisconnectedSubscription();
  const { enqueue } = useSnackbar();

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
    if (listCastDevicesData?.listCastDevices) {
      setCastDevices(
        _.uniqBy(
          listCastDevicesData.listCastDevices.map((x) => ({
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
  }, [data, listDevicesData, listCastDevicesData]);

  useEffect(() => {
    if (deviceConnectedData) {
      enqueue({
        message: `Connected to ${deviceConnectedData.onConnected.name}`,
      });
      if (deviceConnectedData.onConnected.app === "chromecast") {
        setCurrentCastDevice({
          id: deviceConnectedData.onConnected.id,
          type: deviceConnectedData.onConnected.app,
          name: deviceConnectedData.onConnected.name,
          isConnected: true,
        });
        return;
      }
      refetch()
        .then((result) => {
          if (result.data?.connectedDevice) {
            setCurrentDevice({
              id: result.data.connectedDevice.id,
              type: result.data.connectedDevice.app,
              name: result.data.connectedDevice.name,
              isConnected: result.data.connectedDevice.isConnected,
            });
          }
        })
        .catch((e) => console.error(e));
      navigate(0);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [deviceConnectedData]);

  useEffect(() => {
    if (deviceDisconnectedData) {
      enqueue({
        message: `Disconnected from ${deviceDisconnectedData.onDisconnected.name}`,
      });
      refetch().catch((e) => console.error(e));
      setCurrentDevice(undefined);
      navigate(0);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [deviceDisconnectedData]);

  useEffect(() => {
    connectedDeviceData &&
      setCurrentDevice({
        id: connectedDeviceData.connectedDevice.id,
        type: connectedDeviceData.connectedDevice.app,
        name: connectedDeviceData.connectedDevice.name,
        isConnected: connectedDeviceData.connectedDevice.isConnected,
      });
  }, [connectedDeviceData]);

  useEffect(() => {
    connectedCastDeviceData &&
      setCurrentCastDevice({
        id: connectedCastDeviceData.connectedCastDevice.id,
        type: connectedCastDeviceData.connectedCastDevice.app,
        name: connectedCastDeviceData.connectedCastDevice.name,
        isConnected: connectedCastDeviceData.connectedCastDevice.isConnected,
      });
  }, [connectedCastDeviceData]);

  return {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    disconnectFromDevice,
    connectToCastDevice,
    disconnectFromCastDevice,
  };
};
