import _ from "lodash";
import { useEffect, useState } from "react";
import { Device } from "../Types/Device";
import { useListDevicesQuery, useOnNewDeviceSubscription } from "./GraphQL";

export const useDevices = () => {
  const [devices, setDevices] = useState<Device[]>([]);
  const { data } = useOnNewDeviceSubscription();
  const { data: listDevicesData } = useListDevicesQuery();

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
          })),
          "id"
        )
      );
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, listDevicesData]);

  return { devices };
};
