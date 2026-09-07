import { useDevices } from "../../Hooks/useDevices";
import { Button, Dialog, EmptyState, Icons, cn } from "../UI";

export type DeviceDialogProps = {
  isOpen: boolean;
  onClose: () => void;
};

/**
 * The device picker behind the sidebar's status row — the web client's
 * equivalent of the desktop's server switcher.
 *
 * Cast targets and music-player instances are listed together because the
 * choice the user is making is the same either way: where does the sound
 * come out.
 */
const DeviceDialog = ({ isOpen, onClose }: DeviceDialogProps) => {
  const {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    disconnectFromDevice,
    connectToCastDevice,
    disconnectFromCastDevice,
  } = useDevices();

  const rows = [
    ...castDevices.map((device) => ({
      ...device,
      cast: true,
      active: currentCastDevice?.id === device.id,
    })),
    ...devices.map((device) => ({
      ...device,
      cast: false,
      active: currentDevice?.id === device.id,
    })),
  ];

  const connected = !!currentCastDevice || !!currentDevice;

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="Play on"
      icon={Icons.device}
      width={440}
      footer={
        connected ? (
          <Button
            variant="outline"
            onClick={() => {
              if (currentCastDevice) disconnectFromCastDevice();
              if (currentDevice) disconnectFromDevice();
              onClose();
            }}
          >
            Play here instead
          </Button>
        ) : undefined
      }
    >
      {rows.length === 0 ? (
        <EmptyState
          icon={Icons.device}
          title="No other devices found"
          hint="Cast targets and other music-player instances on the network show up here."
        />
      ) : (
        <ul className="flex flex-col gap-1 py-1">
          {rows.map((device) => (
            <li key={`${device.cast ? "cast" : "mp"}-${device.id}`}>
              <button
                type="button"
                onClick={() => {
                  if (device.cast) connectToCastDevice({ id: device.id });
                  else connectToDevice({ id: device.id });
                  onClose();
                }}
                className={cn(
                  "flex h-12 w-full items-center gap-3 rounded-control px-3 text-left",
                  device.active ? "bg-selected" : "hover:bg-hover"
                )}
              >
                <Icons.device
                  size={18}
                  className={device.active ? "text-accent" : "text-dim"}
                />
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-[13px] text-fg">
                    {device.name}
                  </span>
                  <span className="block truncate text-[11px] text-muted">
                    {device.cast ? "Cast" : "music-player"}
                  </span>
                </span>
                {device.active && (
                  <span className="text-[11px] font-semibold text-accent">
                    Playing
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}
    </Dialog>
  );
};

export default DeviceDialog;
