/**
 * A colour per musical key.
 *
 * The point of colouring a key is that **keys which mix look alike**. Scanning a
 * track list for what will mix is a matter of spotting a run of similar
 * colours, which is far faster than reading "8A" and "9A" and comparing them.
 * So the colour comes from the key's position on the circle of fifths — where
 * neighbours are the keys that mix — rather than from its letter name, where
 * neighbours are a semitone apart and sound terrible together.
 *
 * Twelve hues evenly around the wheel. Relative major and minor share a
 * position and so share a hue, which is right — they share a key signature and
 * mix freely — with the minor drawn darker so the two stay distinguishable.
 *
 * The same scheme as the desktop's, deliberately. It is the scheme Mixxx's key
 * colouring is built on rather than a copy of its exact palette, which is
 * compiled into its binary.
 */

/** The colour for a Camelot key such as `"8A"`, or `null` if it is not one. */
export const keyColorFor = (camelot?: string | null): string | null => {
  const parsed = parse(camelot);
  if (!parsed) return null;

  // Position on the wheel, 0..11, straight to a hue: one step round the circle
  // of fifths is 30 degrees.
  const hue = (parsed.number - 1) * 30;
  // Minor darker, so a glance separates 8A from 8B without reading them.
  const lightness = parsed.minor ? 46 : 58;
  return `hsl(${hue} 68% ${lightness}%)`;
};

const parse = (
  camelot?: string | null
): { number: number; minor: boolean } | null => {
  const value = camelot?.trim();
  if (!value || value.length < 2) return null;

  const letter = value.slice(-1).toUpperCase();
  if (letter !== "A" && letter !== "B") return null;

  const number = Number(value.slice(0, -1));
  if (!Number.isInteger(number) || number < 1 || number > 12) return null;

  return { number, minor: letter === "A" };
};
