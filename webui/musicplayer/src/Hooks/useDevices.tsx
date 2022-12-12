import _ from "lodash";
import { useEffect, useState } from "react";
import { Device } from "../Types/Device";
import { useOnNewDeviceSubscription } from "./GraphQL";

export const useDevices = () => {
  const [devices, setDevices] = useState<Device[]>([]);
  const { data } = useOnNewDeviceSubscription();

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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data]);

  return { devices };
};
