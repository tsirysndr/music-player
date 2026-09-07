import { act, render, renderHook, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Provider as JotaiProvider } from "jotai";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { SKINS, SKIN_STORAGE_KEY } from "../State";
import SkinProvider, { useSkin } from "./SkinProvider";

const wrapper = ({ children }: { children: ReactNode }) => (
  <JotaiProvider>
    <SkinProvider>{children}</SkinProvider>
  </JotaiProvider>
);

const setup = () => renderHook(() => useSkin(), { wrapper });

describe("SkinProvider", () => {
  beforeEach(() => {
    localStorage.clear();
    delete document.documentElement.dataset.skin;
  });

  /** Every token in `styles/skins.css` hangs off this attribute. */
  it("puts the skin on the document", () => {
    const { result } = setup();
    expect(document.documentElement.dataset.skin).toBe(result.current.skin);
  });

  it("exposes the skin's display name", () => {
    const { result } = setup();
    act(() => result.current.setSkin("synthwave"));
    expect(result.current.skinName).toBe("Synthwave");
  });

  it("moves the document attribute when the skin changes", () => {
    const { result } = setup();
    act(() => result.current.setSkin("porcelain"));
    expect(document.documentElement.dataset.skin).toBe("porcelain");
  });

  it("persists the choice for the next visit", () => {
    const { result } = setup();
    act(() => result.current.setSkin("neutron"));
    expect(localStorage.getItem(SKIN_STORAGE_KEY)).toBe("neutron");
  });

  it("cycles through the skins and wraps at the end", () => {
    const { result } = setup();

    act(() => result.current.setSkin(SKINS[SKINS.length - 1].id));
    act(() => result.current.cycleSkin());
    expect(result.current.skin).toBe(SKINS[0].id);

    act(() => result.current.cycleSkin());
    expect(result.current.skin).toBe(SKINS[1].id);
  });

  it("visits every skin exactly once per full cycle", () => {
    const { result } = setup();
    act(() => result.current.setSkin(SKINS[0].id));

    const seen = [result.current.skin];
    for (let i = 1; i < SKINS.length; i++) {
      act(() => result.current.cycleSkin());
      seen.push(result.current.skin);
    }

    expect(new Set(seen).size).toBe(SKINS.length);
  });

  it("makes the skin available to any consumer under it", async () => {
    const user = userEvent.setup();

    const Consumer = () => {
      const { skinName, cycleSkin } = useSkin();
      return (
        <button type="button" onClick={cycleSkin}>
          {skinName}
        </button>
      );
    };

    render(
      <JotaiProvider>
        <SkinProvider>
          <Consumer />
        </SkinProvider>
      </JotaiProvider>
    );

    const before = screen.getByRole("button").textContent;
    await user.click(screen.getByRole("button"));
    expect(screen.getByRole("button").textContent).not.toBe(before);
  });
});
