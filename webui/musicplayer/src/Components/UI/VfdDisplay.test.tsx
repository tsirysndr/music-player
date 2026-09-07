import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import VfdDisplay from "./VfdDisplay";

describe("VfdDisplay", () => {
  it("shows the time and the codec line", () => {
    render(
      <VfdDisplay timeText="02:41" infoText="FLAC  1411 kbps  44.1 kHz" />
    );
    expect(screen.getByText("02:41")).toBeInTheDocument();
    // Testing Library collapses runs of whitespace before comparing, so the
    // codec line's double spaces are matched as single ones.
    expect(screen.getByText("FLAC 1411 kbps 44.1 kHz")).toBeInTheDocument();
  });

  /**
   * Stopped is a square, the tape-deck convention the desktop keeps; playing
   * and paused are the transport glyphs. All three are decorative, so the test
   * goes by what is drawn rather than by an accessible name.
   */
  it("draws a square when stopped and a glyph when it is not", () => {
    const { container, rerender } = render(<VfdDisplay stopped />);
    expect(container.querySelector("svg")).toBeNull();

    rerender(<VfdDisplay stopped={false} playing />);
    expect(container.querySelector("svg")).not.toBeNull();
  });

  it("falls back to a blank readout with no track", () => {
    render(<VfdDisplay />);
    expect(screen.getByText("00:00")).toBeInTheDocument();
    expect(screen.getByText("-- kbps --.- kHz")).toBeInTheDocument();
  });

  it("draws two VU strips, one per channel", () => {
    render(<VfdDisplay vuLeft={0.6} vuRight={0.4} />);
    expect(screen.getAllByTestId("meter-strip")).toHaveLength(2);
  });
});
