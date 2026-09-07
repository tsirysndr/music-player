import { atom } from "jotai";
import { Device } from "../Types/Device";

export const devicesAtom = atom<Device[]>([]);
export const castDevicesAtom = atom<Device[]>([]);
export const currentDeviceAtom = atom<Device | undefined>(undefined);
export const currentCastDeviceAtom = atom<Device | undefined>(undefined);

/**
 * Whether the client is talking to a music-player daemon — the desktop's
 * `root.connected`. Fed by the currently-playing poll in `usePlaybackSync`,
 * which is the one request that runs continuously, so it fails the moment the
 * server goes away.
 *
 * Deliberately *not* "is a cast device selected": playing on this device is a
 * perfectly connected state, and the sidebar dot was showing it as an error.
 */
export const serverConnectedAtom = atom(true);
