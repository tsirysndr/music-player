import { useDevices } from "../../Hooks/useDevices";
import { Button, Dialog, Icons, cn, type IconComponent } from "../UI";

export type DeviceDialogProps = {
  isOpen: boolean;
  onClose: () => void;
};

/** How each kind of renderer describes itself. */
const KIND: Record<string, { label: string; icon: IconComponent }> = {
  chromecast: { label: "Chromecast", icon: Icons.chromecast },
  dlna: { label: "UPnP / DLNA", icon: Icons.broadcast },
  "music-player": { label: "music-player", icon: Icons.server },
  xbmc: { label: "Kodi", icon: Icons.device },
};

/**
 * Where the audio comes out — AirPlay-style.
 *
 * A *renderer* is a sink: a Chromecast, a UPnP/DLNA renderer, or another
 * music-player daemon told to play. That is a different question from where
 * the library is *read* from, which is the Servers page; conflating the two is
 * why this dialog used to open from the sidebar's status row.
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

  // Cast targets and peer daemons are both places the sound can come out of,
  // so they are one list rather than two.
  const rows = [
    ...castDevices.map((device) => ({
      ...device,
      cast: true,
      active: currentCastDevice?.id === device.id,
    })),
    ...devices
      .filter((device) => !castDevices.some((cast) => cast.id === device.id))
      .map((device) => ({
        ...device,
        cast: false,
        active: currentDevice?.id === device.id,
      })),
  ];

  const playingElsewhere = !!currentCastDevice || !!currentDevice;

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="Play to"
      icon={Icons.deviceSpeaker}
      width={440}
      footer={
        playingElsewhere ? (
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
      <ul className="flex flex-col gap-1 py-1">
          {/* This machine is always an option, and is where it starts. */}
          <li>
            <button
              type="button"
              onClick={() => {
                if (currentCastDevice) disconnectFromCastDevice();
                if (currentDevice) disconnectFromDevice();
                onClose();
              }}
              className={cn(
                "flex h-12 w-full items-center gap-3 rounded-control px-3 text-left",
                playingElsewhere ? "hover:bg-hover" : "bg-selected"
              )}
            >
              <Icons.volume
                size={18}
                className={playingElsewhere ? "text-dim" : "text-accent"}
              />
              <span className="min-w-0 flex-1">
                <span className="block truncate text-[13px] text-fg">
                  This computer
                </span>
                <span className="block truncate text-[11px] text-muted">
                  Local playback
                </span>
              </span>
              {!playingElsewhere && (
                <span className="text-[11px] font-semibold text-accent">
                  Playing
                </span>
              )}
            </button>
          </li>

          {rows.map((device) => {
            const kind = KIND[device.type] ?? {
              label: device.type,
              icon: Icons.device,
            };
            const Icon = kind.icon;
            return (
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
                  <Icon
                    size={18}
                    className={device.active ? "text-accent" : "text-dim"}
                  />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-[13px] text-fg">
                      {device.name}
                    </span>
                    <span className="block truncate text-[11px] text-muted">
                      {kind.label}
                    </span>
                  </span>
                  {device.active && (
                    <span className="text-[11px] font-semibold text-accent">
                      Playing
                    </span>
                  )}
                </button>
              </li>
            );
          })}
        </ul>
      {/* This machine is always there, so an empty network is a note under the
          list rather than an empty state instead of it. */}
      {rows.length === 0 && (
        <p className="px-3 pb-1 pt-2 text-[11px] text-muted">
          Chromecasts, UPnP/DLNA renderers and other music-player instances on
          this network show up here.
        </p>
      )}
    </Dialog>
  );
};

export default DeviceDialog;
