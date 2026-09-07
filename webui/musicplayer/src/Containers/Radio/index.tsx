import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { AppShell } from "../../Components/Layout";
import {
  Button,
  Dialog,
  EmptyState,
  FilterBox,
  IconButton,
  Icons,
  RadioCategoryTile,
  RadioRow,
  RadioSkeletonList,
  cn,
} from "../../Components/UI";
import AddStationForm from "./AddStationForm";
import { gql, STATION_FIELDS, type Station } from "./api";
import { RADIO_CATEGORIES } from "./categories";

type Tab = "search" | "saved" | "stations";

const TABS: { key: Tab; label: string }[] = [
  { key: "search", label: "Search" },
  { key: "saved", label: "Bookmarked" },
  { key: "stations", label: "Stations" },
];

/**
 * Internet radio, laid out as the desktop client lays it out: three tabs, a
 * category grid behind the first, and a quick filter over whichever list is
 * on screen.
 */
export default function RadioPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [tab, setTab] = useState<Tab>("search");
  const [stations, setStations] = useState<Station[]>([]);
  const [saved, setSaved] = useState<Station[]>([]);
  const [mine, setMine] = useState<Station[]>([]);
  const [savedLoading, setSavedLoading] = useState(true);
  const [mineLoading, setMineLoading] = useState(true);
  const [addOpen, setAddOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("");
  const [category, setCategory] = useState<{ label: string; term: string }>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  // Only the newest discover call may write the list: typing leaves several in
  // flight and a slow early one must not overwrite a later result.
  const requestId = useRef(0);

  const loadSaved = useCallback(async () => {
    setSavedLoading(true);
    try {
      const data = await gql(`query { savedRadios { ${STATION_FIELDS} } }`);
      setSaved(data.savedRadios || []);
    } finally {
      setSavedLoading(false);
    }
  }, []);

  const loadMine = useCallback(async () => {
    setMineLoading(true);
    try {
      const data = await gql(`query { radioStations { ${STATION_FIELDS} } }`);
      setMine(data.radioStations || []);
    } finally {
      setMineLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSaved();
    loadMine();
  }, [loadSaved, loadMine]);

  // `r` from anywhere lands here with the search sheet open, matching the
  // desktop shortcut.
  useEffect(() => {
    if (searchParams.get("search") === "1") setSearchOpen(true);
  }, [searchParams]);

  const discover = async (term?: string, search?: string) => {
    const id = ++requestId.current;
    setLoading(true);
    setError("");
    try {
      const data = await gql(
        `query($query:String,$category:String){ radios(query:$query,category:$category){${STATION_FIELDS}}}`,
        { query: search, category: term }
      );
      if (id === requestId.current) setStations(data.radios || []);
    } catch (e) {
      if (id === requestId.current) {
        setStations([]);
        setError(
          e instanceof Error ? e.message : "Unable to load radio stations"
        );
      }
    } finally {
      if (id === requestId.current) setLoading(false);
    }
  };

  const openCategory = (label: string, term: string) => {
    setCategory({ label, term });
    setStations([]);
    setFilter("");
    discover(term);
  };

  const play = (station: Station) =>
    gql(`mutation($station:RadioStationInput!){playRadio(station:$station)}`, {
      station,
    });

  const toggleBookmark = async (station: Station) => {
    const exists = saved.some((entry) => entry.id === station.id);
    await gql(
      exists
        ? `mutation($id:String!){removeSavedRadio(id:$id)}`
        : `mutation($station:RadioStationInput!){saveRadio(station:$station)}`,
      exists ? { id: station.id } : { station }
    );
    // Unbookmarking one of the user's own stations drops it from that list too
    // — locally the two are the same row.
    await Promise.all([loadSaved(), loadMine()]);
  };

  const closeSearch = () => {
    setSearchOpen(false);
    if (searchParams.has("search")) setSearchParams({}, { replace: true });
  };

  /** The list on screen, after the quick filter. */
  const visible = useMemo(() => {
    const source = category
      ? stations
      : tab === "saved"
        ? saved
        : tab === "stations"
          ? mine
          : stations;
    const needle = filter.trim().toLowerCase();
    if (!needle) return source;
    return source.filter((station) =>
      [station.name, station.genre, station.country, station.source].some(
        (value) => value?.toLowerCase().includes(needle)
      )
    );
  }, [category, stations, tab, saved, mine, filter]);

  const listLoading = category
    ? loading
    : tab === "saved"
      ? savedLoading
      : tab === "stations"
        ? mineLoading
        : loading;

  const bookmarked = (station: Station) =>
    saved.some((entry) => entry.id === station.id);

  const rows = (items: Station[]) =>
    items.map((station) => (
      <RadioRow
        key={station.id}
        station={{
          id: station.id,
          name: station.name,
          subtitle: [station.genre, station.country]
            .filter(Boolean)
            .join(" · "),
          source: station.source,
          logo: station.logo,
          bookmarked: bookmarked(station),
        }}
        onPlay={() => play(station)}
        onBookmark={() => toggleBookmark(station)}
      />
    ));

  return (
    <AppShell
      title="Internet Radio"
      onBack={category ? () => setCategory(undefined) : undefined}
    >
      {category ? (
        <>
          <div className="mb-3 flex items-center gap-3">
            <h2 className="flex-1 truncate text-base font-bold text-fg">
              {category.label}
            </h2>
            <FilterBox
              value={filter}
              placeholder="Filter stations…"
              className="w-full max-w-[190px]"
              onChange={setFilter}
            />
          </div>
          {loading ? (
            <RadioSkeletonList />
          ) : error ? (
            <EmptyState
              icon={Icons.broadcast}
              title="Unable to load stations"
              hint={error}
            />
          ) : visible.length === 0 ? (
            <EmptyState
              icon={Icons.broadcast}
              title="No station found in this category"
            />
          ) : (
            <div className="flex flex-col">{rows(visible)}</div>
          )}
        </>
      ) : (
        <>
          <div className="mb-4 flex items-center gap-2 border-b border-line">
            {TABS.map((entry) => (
              <button
                key={entry.key}
                type="button"
                onClick={() => {
                  setTab(entry.key);
                  setFilter("");
                }}
                className={cn(
                  "-mb-px border-b-2 px-3 pb-2 text-xs font-semibold transition-colors",
                  tab === entry.key
                    ? "border-accent text-accent"
                    : "border-transparent text-dim hover:text-fg"
                )}
              >
                {entry.label}
              </button>
            ))}
            <div className="ml-auto flex items-center gap-2 pb-1">
              {tab !== "search" && (
                <FilterBox
                  value={filter}
                  placeholder="Filter stations…"
                  className="w-[190px]"
                  onChange={setFilter}
                />
              )}
              {tab === "stations" && (
                <IconButton
                  icon={Icons.circlePlus}
                  iconSize={17}
                  aria-label="Add a station"
                  onClick={() => setAddOpen(true)}
                />
              )}
            </div>
          </div>

          {tab === "search" && (
            <>
              <button
                type="button"
                onClick={() => setSearchOpen(true)}
                className="mb-4 flex h-11 w-full items-center gap-[10px] border-b border-line px-1 text-left"
              >
                <Icons.search size={19} className="text-muted" />
                <span className="text-sm text-muted">
                  Search radio stations…
                </span>
              </button>
              <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 lg:grid-cols-4">
                {RADIO_CATEGORIES.map((entry) => (
                  <RadioCategoryTile
                    key={entry.label}
                    label={entry.label}
                    term={entry.term}
                    icon={entry.icon}
                    color={entry.color}
                    onSelect={openCategory}
                  />
                ))}
              </div>
            </>
          )}

          {tab !== "search" &&
            (listLoading ? (
              <RadioSkeletonList />
            ) : visible.length === 0 ? (
              <EmptyState
                icon={Icons.broadcast}
                title={
                  filter
                    ? `Nothing matches “${filter}”`
                    : tab === "stations"
                      ? "No station of your own yet"
                      : "No bookmarked station yet"
                }
                hint={
                  filter || tab !== "stations"
                    ? undefined
                    : "Add one with its stream url."
                }
                action={
                  tab === "stations" && !filter ? (
                    <Button
                      icon={Icons.circlePlus}
                      onClick={() => setAddOpen(true)}
                    >
                      Add a station
                    </Button>
                  ) : undefined
                }
              />
            ) : (
              <div className="flex flex-col">{rows(visible)}</div>
            ))}
        </>
      )}

      <AddStationForm
        isOpen={addOpen}
        onClose={() => setAddOpen(false)}
        onAdded={() => {
          loadSaved();
          loadMine();
        }}
      />

      <Dialog
        isOpen={searchOpen}
        onClose={closeSearch}
        title="Search radio stations"
        icon={Icons.search}
        width={620}
      >
        <FilterBox
          value={query}
          autoFocus
          placeholder="Search radio stations…"
          className="mb-3"
          onChange={(value) => {
            setQuery(value);
            if (value.trim()) {
              discover(undefined, value);
            } else {
              // Bump the id so an in-flight response cannot repopulate the
              // list the user just cleared.
              requestId.current++;
              setStations([]);
              setLoading(false);
            }
          }}
        />
        {loading ? (
          <RadioSkeletonList rows={5} />
        ) : error ? (
          <p className="py-6 text-center text-xs text-syntax-error">{error}</p>
        ) : stations.length === 0 ? (
          <p className="py-6 text-center text-xs text-muted">
            {query.trim() ? "No station found" : "Type to search"}
          </p>
        ) : (
          <div className="flex flex-col">
            {stations.map((station) => (
              <RadioRow
                key={station.id}
                station={{
                  id: station.id,
                  name: station.name,
                  subtitle: [station.genre, station.country]
                    .filter(Boolean)
                    .join(" · "),
                  source: station.source,
                  logo: station.logo,
                  bookmarked: bookmarked(station),
                }}
                onPlay={() => {
                  play(station);
                  closeSearch();
                }}
                onBookmark={() => toggleBookmark(station)}
              />
            ))}
          </div>
        )}
      </Dialog>
    </AppShell>
  );
}
