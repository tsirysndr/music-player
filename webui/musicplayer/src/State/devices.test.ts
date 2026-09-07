import { createStore } from "jotai";
import { describe, expect, it } from "vitest";
import type { Device } from "../Types/Device";
import {
  castDevicesAtom,
  currentCastDeviceAtom,
  currentDeviceAtom,
  devicesAtom,
} from "./devices";

const device = (id: string, name: string, type = "music-player"): Device => ({
  id,
  name,
  type,
  isConnected: false,
});

describe("device atoms", () => {
  it("start empty and unconnected", () => {
    const store = createStore();
    expect(store.get(devicesAtom)).toEqual([]);
    expect(store.get(castDevicesAtom)).toEqual([]);
    expect(store.get(currentDeviceAtom)).toBeUndefined();
    expect(store.get(currentCastDeviceAtom)).toBeUndefined();
  });

  it("hold the discovered devices", () => {
    const store = createStore();
    store.set(devicesAtom, [device("1", "Studio")]);
    store.set(castDevicesAtom, [device("2", "Living room", "chromecast")]);

    expect(store.get(devicesAtom)).toHaveLength(1);
    expect(store.get(castDevicesAtom)[0].type).toBe("chromecast");
  });

  /**
   * mDNS re-announces, so the sync hook appends and dedupes by id. The atom
   * itself only has to survive being rewritten in place.
   */
  it("survives a device being appended by an announcement", () => {
    const store = createStore();
    store.set(devicesAtom, [device("1", "Studio")]);
    store.set(devicesAtom, (current) => [...current, device("2", "Kitchen")]);

    expect(store.get(devicesAtom).map((entry) => entry.id)).toEqual(["1", "2"]);
  });

  /**
   * The two current-device atoms are separate on purpose: a cast target and a
   * music-player instance are different kinds of connection, and connecting to
   * one must not silently clear the other.
   */
  it("tracks the connected cast target and daemon independently", () => {
    const store = createStore();
    store.set(currentDeviceAtom, device("1", "Studio"));
    store.set(
      currentCastDeviceAtom,
      device("2", "Living room", "chromecast")
    );

    store.set(currentCastDeviceAtom, undefined);
    expect(store.get(currentCastDeviceAtom)).toBeUndefined();
    expect(store.get(currentDeviceAtom)?.name).toBe("Studio");
  });

  it("clears back to playing locally on disconnect", () => {
    const store = createStore();
    store.set(currentDeviceAtom, device("1", "Studio"));
    store.set(currentDeviceAtom, undefined);
    expect(store.get(currentDeviceAtom)).toBeUndefined();
  });
});
