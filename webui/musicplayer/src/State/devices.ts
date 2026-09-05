import { atom } from "jotai";
import { Device } from "../Types/Device";

export const devicesAtom = atom<Device[]>([]);
export const castDevicesAtom = atom<Device[]>([]);
export const currentDeviceAtom = atom<Device | undefined>(undefined);
export const currentCastDeviceAtom = atom<Device | undefined>(undefined);
