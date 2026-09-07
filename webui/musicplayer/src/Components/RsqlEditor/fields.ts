/**
 * The track fields a smart-playlist filter may mention.
 *
 * Mirrors `rsql/src/schema.rs` on the Rust side. Kept as a literal rather than
 * fetched so the editor colours correctly on first paint, with no request in
 * the way; the daemon validates for real when the filter is saved, so a drift
 * here shows up as a red field the daemon then accepts, not as a wrong result.
 */

export type FieldType = "text" | "integer" | "boolean" | "timestamp";

export type FieldDef = {
  name: string;
  label: string;
  type: FieldType;
};

export const FIELDS: FieldDef[] = [
  { name: "title", label: "Title", type: "text" },
  { name: "artist", label: "Artist", type: "text" },
  { name: "album", label: "Album", type: "text" },
  { name: "genre", label: "Genre", type: "text" },
  { name: "year", label: "Year", type: "integer" },
  { name: "track", label: "Track number", type: "integer" },
  { name: "duration", label: "Duration", type: "integer" },
  { name: "bitrate", label: "Bitrate", type: "integer" },
  { name: "samplerate", label: "Sample rate", type: "integer" },
  { name: "uri", label: "File path", type: "text" },
  { name: "liked", label: "Liked", type: "boolean" },
  { name: "playcount", label: "Play count", type: "integer" },
  { name: "skipcount", label: "Skip count", type: "integer" },
  { name: "lastplayed", label: "Last played", type: "timestamp" },
  { name: "added", label: "Date added", type: "timestamp" },
];

export const TRACK_FIELDS = new Set(FIELDS.map((field) => field.name));

export const fieldByName = (name: string): FieldDef | undefined =>
  FIELDS.find((field) => field.name === name.toLowerCase());

/**
 * Operators worth offering for a field's type. Ordering comparisons mean
 * nothing for text and `=like=` means nothing for a number — offering them
 * would teach the wrong thing.
 */
export function opsFor(type: FieldType | undefined): {
  op: string;
  hint: string;
}[] {
  const common = [
    { op: "==", hint: "equals" },
    { op: "!=", hint: "does not equal" },
  ];
  const tail = [
    { op: "=in=", hint: "any of" },
    { op: "=out=", hint: "none of" },
    { op: "=null=", hint: "has no value" },
    { op: "=notnull=", hint: "has a value" },
  ];
  if (type === "text" || type === undefined) {
    return [
      ...common,
      { op: "=like=", hint: "contains — * is a wildcard" },
      ...tail,
    ];
  }
  return [
    ...common,
    { op: ">", hint: "greater than" },
    { op: ">=", hint: "at least" },
    { op: "<", hint: "less than" },
    { op: "<=", hint: "at most" },
    ...tail,
  ];
}

/**
 * Values worth suggesting. Only where a fixed vocabulary exists — nobody can
 * guess an artist name, but `true`/`false` and relative ages are finite and
 * easy to get wrong.
 */
export function valuesFor(field: FieldDef | undefined): {
  text: string;
  hint: string;
}[] {
  if (!field) return [];
  if (field.type === "boolean") {
    return [
      { text: "true", hint: "" },
      { text: "false", hint: "" },
    ];
  }
  if (field.type === "timestamp") {
    return [
      { text: "7d", hint: "in the last week" },
      { text: "30d", hint: "in the last 30 days" },
      { text: "6m", hint: "in the last 6 months" },
      { text: "1y", hint: "in the last year" },
      { text: "2024-01-01", hint: "on or after a date" },
    ];
  }
  return [];
}
