import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { graphql, HttpResponse } from "msw";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { makeQueryClient } from "../test/render";
import { server } from "../test/server";
import { usePlayTrack } from "./usePlayTrack";

const TRACK = "e5f9895f3560dce3181eff25ca349f43";

const setup = () => {
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={makeQueryClient()}>
      {children}
    </QueryClientProvider>
  );

  return renderHook(() => usePlayTrack(), { wrapper });
};

describe("usePlayTrack", () => {
  it("requests atomic playback for the track it was given", async () => {
    const seen = vi.fn();
    const unexpected = vi.fn();
    server.use(
      graphql.mutation("PlayTrack", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playTrack: true } });
      }),
      graphql.mutation(/^(PlayNext|Next|Play)$/, () => {
        unexpected();
        return HttpResponse.json({ errors: [{ message: "Unexpected legacy playback command" }] });
      }),
    );

    const { result } = setup();
    await act(async () => {
      await result.current(TRACK);
    });

    expect(seen).toHaveBeenCalledWith({ trackId: TRACK });
    expect(unexpected).not.toHaveBeenCalled();
  });
});
