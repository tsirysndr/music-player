import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterAll, afterEach, beforeAll, vi } from "vitest";
import { server } from "./server";

/**
 * Test environment setup, run before every test file.
 *
 * The MSW server is started here rather than per-file so a test only has to
 * declare the handlers it wants to *override*; anything it does not mention
 * falls through to the defaults in `handlers.ts`. `onUnhandledRequest: "error"`
 * is deliberate — a request nobody stubbed is almost always a test asserting
 * against a loading state it did not mean to be in.
 */
beforeAll(() => server.listen({ onUnhandledRequest: "error" }));
afterEach(() => {
  server.resetHandlers();
  cleanup();
  localStorage.clear();
});
afterAll(() => server.close());

/**
 * jsdom implements neither of these, and both are load-bearing:
 * `ResizeObserver` is what `MarqueeText` measures with, and
 * `matchMedia` is what the shell asks before closing the queue on a phone.
 */
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

vi.stubGlobal("ResizeObserver", ResizeObserverStub);

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
});

// React Aria (under HeroUI's overlays) measures the scrollbar and calls this
// on any element it moves focus to; jsdom has no layout, so it is a no-op.
if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = () => {};
}

// jsdom has no pointer capture, which React Aria's press handling asks for.
if (!Element.prototype.hasPointerCapture) {
  Element.prototype.hasPointerCapture = () => false;
  Element.prototype.setPointerCapture = () => {};
  Element.prototype.releasePointerCapture = () => {};
}
