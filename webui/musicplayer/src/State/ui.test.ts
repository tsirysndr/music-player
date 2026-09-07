import { createStore } from "jotai";
import { describe, expect, it } from "vitest";
import {
  fullPlayerOpenAtom,
  mobileMenuOpenAtom,
  paletteOpenAtom,
  queueOpenAtom,
  sidebarOpenAtom,
} from "./ui";

describe("chrome atoms", () => {
  /** The sidebar is open by default, as it is on the desktop; nothing else is. */
  it("open the sidebar and nothing else", () => {
    const store = createStore();
    expect(store.get(sidebarOpenAtom)).toBe(true);
    expect(store.get(queueOpenAtom)).toBe(false);
    expect(store.get(paletteOpenAtom)).toBe(false);
    expect(store.get(fullPlayerOpenAtom)).toBe(false);
    expect(store.get(mobileMenuOpenAtom)).toBe(false);
  });

  it("toggle without reading each other", () => {
    const store = createStore();
    store.set(queueOpenAtom, (open) => !open);
    expect(store.get(queueOpenAtom)).toBe(true);
    expect(store.get(sidebarOpenAtom)).toBe(true);

    store.set(sidebarOpenAtom, (open) => !open);
    expect(store.get(sidebarOpenAtom)).toBe(false);
    expect(store.get(queueOpenAtom)).toBe(true);
  });

  /**
   * The queue drawer and the full player can both be open — the drawer is
   * beside the content, the player covers it — so neither closes the other.
   */
  it("let the queue and the full player be open together", () => {
    const store = createStore();
    store.set(queueOpenAtom, true);
    store.set(fullPlayerOpenAtom, true);

    expect(store.get(queueOpenAtom)).toBe(true);
    expect(store.get(fullPlayerOpenAtom)).toBe(true);
  });
});
