import { useSnackbar } from "baseui/snackbar";
import _ from "lodash";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useSetAtom } from "jotai";
import {
  castDevicesAtom,
  currentCastDeviceAtom,
  currentDeviceAtom,
  devicesAtom,
} from "../State";
import {
  OnDeviceConnectedDocument,
  OnDeviceConnectedSubscription,
  OnDeviceDisconnectedDocument,
  OnDeviceDisconnectedSubscription,
  OnNewDeviceDocument,
  OnNewDeviceSubscription,
  useConnectedCastDeviceQuery,
  useConnectedDeviceQuery,
  useListCastDevicesQuery,
  useListDevicesQuery,
} from "./GraphQL";
import { useGraphQLSubscription } from "./useGraphQLSubscription";

/**
 * Keeps the device atoms in sync with the server. Should be mounted once
 * (see AppStateSync).
 */
export const useDevicesSync = () => {
  const navigate = useNavigate();
  const { enqueue } = useSnackbar();
  const setDevices = useSetAtom(devicesAtom);
  const setCastDevices = useSetAtom(castDevicesAtom);
  const setCurrentDevice = useSetAtom(currentDeviceAtom);
  const setCurrentCastDevice = useSetAtom(currentCastDeviceAtom);

  const { data: listDevicesData } = useListDevicesQuery();
  const { data: listCastDevicesData } = useListCastDevicesQuery();
  const { data: connectedDeviceData, refetch } = useConnectedDeviceQuery();
  const { data: connectedCastDeviceData } = useConnectedCastDeviceQuery();

  useGraphQLSubscription<OnNewDeviceSubscription>(
    OnNewDeviceDocument,
    undefined,
    (data) => {
      if (
        data.onNewDevice &&
        (data.onNewDevice.app === "music-player" ||
          data.onNewDevice.app === "xbmc")
      ) {
        setDevices((devices) =>
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
    }
  );

  useGraphQLSubscription<OnDeviceConnectedSubscription>(
    OnDeviceConnectedDocument,
    undefined,
    (deviceConnectedData) => {
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
  );

  useGraphQLSubscription<OnDeviceDisconnectedSubscription>(
    OnDeviceDisconnectedDocument,
    undefined,
    (deviceDisconnectedData) => {
      enqueue({
        message: `Disconnected from ${deviceDisconnectedData.onDisconnected.name}`,
      });
      refetch().catch((e) => console.error(e));
      setCurrentDevice(undefined);
      navigate(0);
    }
  );

  useEffect(() => {
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
  }, [listDevicesData, listCastDevicesData, setDevices, setCastDevices]);

  useEffect(() => {
    connectedDeviceData &&
      setCurrentDevice({
        id: connectedDeviceData.connectedDevice.id,
        type: connectedDeviceData.connectedDevice.app,
        name: connectedDeviceData.connectedDevice.name,
        isConnected: connectedDeviceData.connectedDevice.isConnected,
      });
  }, [connectedDeviceData, setCurrentDevice]);

  useEffect(() => {
    connectedCastDeviceData &&
      setCurrentCastDevice({
        id: connectedCastDeviceData.connectedCastDevice.id,
        type: connectedCastDeviceData.connectedCastDevice.app,
        name: connectedCastDeviceData.connectedCastDevice.name,
        isConnected: connectedCastDeviceData.connectedCastDevice.isConnected,
      });
  }, [connectedCastDeviceData, setCurrentCastDevice]);
};
