import styled from "@emotion/styled";
import { useTheme } from "@emotion/react";
import { FC, useEffect, useRef, useState } from "react";
import ControlBar from "../../Components/ControlBar";
import Sidebar from "../../Components/Sidebar/SidebarWithData";
import RadioArt from "../../Components/RadioArt";
import Heart from "../../Components/Icons/Heart";
import HeartOutline from "../../Components/Icons/HeartOutline";
import { fetcher } from "../../Api/fetcher";
import {
  Activity,
  Disc,
  Globe,
  Headphones,
  Mic,
  Music,
  Radio,
  Search as SearchIcon,
  Smile,
  Speaker,
  Zap,
} from "@styled-icons/feather";
import { Tabs, Tab } from "baseui/tabs-motion";
import ContentLoader from "react-content-loader";
import { useSearchParams } from "react-router-dom";

type Station = {
  id: string;
  name: string;
  streamUrl: string;
  source: string;
  genre: string;
  country: string;
  logo: string;
  bitrate: number;
};

const categories = [
  { label: "Synthwave", term: "synthwave", Icon: Activity, color: "#ff4fa3" },
  { label: "Lo-fi", term: "lofi", Icon: Headphones, color: "#39d9e8" },
  { label: "Jazz", term: "jazz", Icon: Music, color: "#f2c94c" },
  { label: "Techno", term: "techno", Icon: Speaker, color: "#9b6cff" },
  { label: "Ambient", term: "ambient", Icon: Activity, color: "#39d9e8" },
  { label: "Classical", term: "classical", Icon: Music, color: "#f052d4" },
  { label: "Rock", term: "rock", Icon: Zap, color: "#ff4fa3" },
  { label: "Pop", term: "pop", Icon: Mic, color: "#f052d4" },
  { label: "Electronic", term: "electronic", Icon: Disc, color: "#4d8dff" },
  { label: "Hip-Hop", term: "hip hop", Icon: Disc, color: "#f2c94c" },
  { label: "Chillout", term: "chill", Icon: Smile, color: "#39d9e8" },
  { label: "Dance", term: "dance", Icon: Speaker, color: "#ff4fa3" },
  { label: "Reggae", term: "reggae", Icon: Music, color: "#f2c94c" },
  { label: "Metal", term: "metal", Icon: Zap, color: "#9b6cff" },
  { label: "News", term: "news", Icon: Radio, color: "#39d9e8" },
  { label: "World", term: "world", Icon: Globe, color: "#4d8dff" },
  { label: "House", term: "house", Icon: Speaker, color: "#39d9e8" },
  { label: "Trance", term: "trance", Icon: Activity, color: "#9b6cff" },
  {
    label: "Drum & Bass",
    term: "drum and bass",
    Icon: Speaker,
    color: "#4d8dff",
  },
  { label: "Disco", term: "disco", Icon: Disc, color: "#ff4fa3" },
  { label: "Funk", term: "funk", Icon: Disc, color: "#f052d4" },
  { label: "Soul", term: "soul", Icon: Music, color: "#ff4fa3" },
  { label: "R&B", term: "r&b", Icon: Mic, color: "#9b6cff" },
  { label: "Blues", term: "blues", Icon: Music, color: "#4d8dff" },
  { label: "Country", term: "country", Icon: Music, color: "#f2c94c" },
  { label: "Folk", term: "folk", Icon: Music, color: "#39d9e8" },
  { label: "Punk", term: "punk", Icon: Zap, color: "#ff4fa3" },
  { label: "Indie", term: "indie", Icon: Headphones, color: "#f052d4" },
  { label: "Latin", term: "latin", Icon: Music, color: "#f052d4" },
  { label: "K-Pop", term: "k-pop", Icon: Mic, color: "#ff4fa3" },
  { label: "Gospel", term: "gospel", Icon: Music, color: "#f2c94c" },
  { label: "Oldies", term: "oldies", Icon: Disc, color: "#39d9e8" },
  { label: "Soundtrack", term: "soundtrack", Icon: Headphones, color: "#4d8dff" },
];

// index.css pins `body { overflow-y: hidden }`, so the document never scrolls
// and a page has to bring its own scroller. This one is the whole page: the
// shell fills the viewport and scrolls everything inside it.
const Shell = styled.div`
  display: flex;
  background: ${(props) => props.theme.colors.background};
  height: 100vh;
  overflow-y: auto;
  font-family: RockfordSansRegular;

  &::-webkit-scrollbar {
    display: none;
  }
  scrollbar-width: none;
  -ms-overflow-style: none;

  & button,
  & input {
    font-family: inherit;
  }
`;

const Main = styled.main`
  flex: 1;
  min-width: 0;
`;

const Body = styled.div`
  padding: 24px 32px 130px;
`;

const PageTitle = styled.h1`
  color: ${(props) => props.theme.colors.text};
  font-family: RockfordSansBold;
  font-size: 24px;
`;

const Grid = styled.div`
  display: grid;
  grid-template-columns: repeat(4, minmax(120px, 1fr));
  gap: 10px;
  margin-bottom: 20px;
`;

const Category = styled.button`
  height: 62px;
  border: 1px solid ${(props) => props.theme.colors.separator};
  border-radius: 8px;
  cursor: pointer;
  color: ${(props) => props.theme.colors.text};
  background: ${(props) => props.theme.colors.secondaryBackground};
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 14px;
  text-align: left;

  &:hover {
    border-color: #ab28fc99;
  }
`;

const CategoryIcon = styled.span<{ color: string }>`
  width: 34px;
  height: 34px;
  flex: 0 0 34px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: ${(props) => props.theme.colors.cover};
  color: ${(props) => props.color};
`;

const Row = styled.div`
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px;
  border-bottom: 1px solid ${(props) => props.theme.colors.separator};
  color: ${(props) => props.theme.colors.text};
`;

const Info = styled.div`
  flex: 1;
  min-width: 0;
  overflow: hidden;
`;

const Name = styled.div`
  font-family: RockfordSansBold;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
`;

const Meta = styled.div`
  color: ${(props) => props.theme.colors.secondaryText};
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
`;

const Action = styled.button`
  border: 0;
  background: transparent;
  color: #ab28fc;
  cursor: pointer;
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;

  &:hover {
    opacity: 0.6;
  }
`;

const Modal = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.67);
  z-index: 20;
  display: flex;
  justify-content: center;
  padding-top: 80px;
`;

const Panel = styled.div`
  width: min(620px, calc(100vw - 80px));
  height: min(520px, calc(100vh - 160px));
  background: ${(props) => props.theme.colors.popoverBackground};
  border-radius: 14px;
  padding: 18px;
  overflow: auto;
`;

const SearchField = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 2px 14px;
  margin-bottom: 8px;
  border-bottom: 1px solid ${(props) => props.theme.colors.separator};
  color: ${(props) => props.theme.colors.secondaryText};
`;

const SearchLauncher = styled.button`
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 14px 0 20px;
  padding: 12px 2px;
  border: 0;
  border-bottom: 1px solid ${(props) => props.theme.colors.separator};
  background: transparent;
  color: ${(props) => props.theme.colors.secondaryText};
  font-size: 16px;
  text-align: left;
  cursor: text;
`;

const Input = styled.input`
  flex: 1;
  min-width: 0;
  padding: 8px 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: ${(props) => props.theme.colors.text};
  font-size: 17px;
`;

const CategoryHeader = styled.div`
  display: flex;
  align-items: center;
  gap: 14px;
  margin: 10px 0 20px;
  color: ${(props) => props.theme.colors.text};
`;

const CategoryTitle = styled.h2`
  color: ${(props) => props.theme.colors.text};
  font-family: RockfordSansBold;
  font-size: 18px;
`;

const BackButton = styled.button`
  border: 0;
  background: transparent;
  color: ${(props) => props.theme.colors.text};
  font-size: 28px;
  line-height: 1;
  cursor: pointer;
  padding: 4px 8px;
`;

const ErrorText = styled.div`
  padding: 24px 0;
  color: #d45769;
`;

// ── Add-station form ────────────────────────────────────────────────────────

const FormPanel = styled.div`
  width: min(520px, calc(100vw - 80px));
  max-height: calc(100vh - 160px);
  background: ${(props) => props.theme.colors.popoverBackground};
  border-radius: 14px;
  padding: 22px 24px 18px;
  overflow: auto;
  color: ${(props) => props.theme.colors.text};
`;

const FormTitle = styled.h2`
  font-family: RockfordSansBold;
  font-size: 18px;
  margin: 0 0 4px;
`;

const FormHint = styled.p`
  color: ${(props) => props.theme.colors.secondaryText};
  font-size: 13px;
  margin: 0 0 18px;
`;

const Field = styled.label`
  display: block;
  margin-bottom: 14px;
`;

const FieldLabel = styled.span`
  display: block;
  font-size: 12px;
  color: ${(props) => props.theme.colors.secondaryText};
  margin-bottom: 5px;
`;

const FieldInput = styled.input`
  width: 100%;
  box-sizing: border-box;
  padding: 9px 11px;
  border-radius: 7px;
  border: 1px solid ${(props) => props.theme.colors.separator};
  background: ${(props) => props.theme.colors.secondaryBackground};
  color: ${(props) => props.theme.colors.text};
  font-size: 14px;
  outline: 0;

  &:focus {
    border-color: #ab28fc;
  }
`;

const FormActions = styled.div`
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 20px;
`;

const Button = styled.button<{ primary?: boolean }>`
  padding: 9px 18px;
  border-radius: 7px;
  font-size: 14px;
  cursor: pointer;
  border: 1px solid
    ${(props) => (props.primary ? "#ab28fc" : props.theme.colors.separator)};
  background: ${(props) => (props.primary ? "#ab28fc" : "transparent")};
  color: ${(props) => (props.primary ? "#fff" : props.theme.colors.text)};

  &:disabled {
    opacity: 0.5;
    cursor: default;
  }
`;

const Status = styled.div<{ tone: "ok" | "error" | "muted" }>`
  font-size: 12px;
  min-height: 16px;
  margin-top: -8px;
  margin-bottom: 12px;
  color: ${(props) =>
    props.tone === "error"
      ? "#d45769"
      : props.tone === "ok"
      ? "#2fbf71"
      : props.theme.colors.secondaryText};
`;

const AddStationButton = styled.button`
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 14px 0 4px;
  padding: 9px 14px;
  border-radius: 7px;
  border: 1px dashed ${(props) => props.theme.colors.separator};
  background: transparent;
  color: ${(props) => props.theme.colors.text};
  font-size: 14px;
  cursor: pointer;

  &:hover {
    border-color: #ab28fc99;
  }
`;

const EmptyText = styled.div`
  padding: 24px 0;
  color: ${(props) => props.theme.colors.secondaryText};
`;

export type StationLoaderProps = {
  rows?: number;
};

/**
 * Skeleton rows shown while stations are being fetched. The colors come from
 * the active theme so the animation reads correctly in light and dark.
 */
const StationLoader: FC<StationLoaderProps> = ({ rows }) => {
  const theme = useTheme();
  return (
    <div>
      {Array.from({ length: rows! }).map((_, i) => (
        <ContentLoader
          key={i}
          speed={1.6}
          width="100%"
          height={65}
          viewBox="0 0 700 65"
          backgroundColor={theme.colors.loaderBackground}
          foregroundColor={theme.colors.loaderForeground}
        >
          <rect x="0" y="10" rx="6" ry="6" width="44" height="44" />
          <rect x="60" y="15" rx="4" ry="4" width="52%" height="13" />
          <rect x="60" y="38" rx="3" ry="3" width="34%" height="9" />
        </ContentLoader>
      ))}
    </div>
  );
};

StationLoader.defaultProps = {
  rows: 6,
};

async function gql(query: string, variables: any = {}) {
  return fetcher<any, any>(query, variables)();
}

const fields = `id name streamUrl source genre country logo bitrate`;

export type AddStationProps = {
  onClose: () => void;
  onAdded: (station: Station) => void;
};

/**
 * The "add station" form. The stream url is checked against the station itself
 * before anything is saved — an unreachable url is the one mistake that makes a
 * station useless — and a station that announces itself over ICY fills in its
 * own name, genre and bitrate.
 */
const AddStationForm: FC<AddStationProps> = ({ onClose, onAdded }) => {
  const [name, setName] = useState("");
  const [streamUrl, setStreamUrl] = useState("");
  const [genre, setGenre] = useState("");
  const [country, setCountry] = useState("");
  const [logo, setLogo] = useState("");
  const [checking, setChecking] = useState(false);
  const [saving, setSaving] = useState(false);
  const [status, setStatus] = useState<{
    tone: "ok" | "error" | "muted";
    text: string;
  }>({ tone: "muted", text: "" });
  // Only the newest check may write the status: a url typed quickly leaves
  // several in flight, and a slow early one must not overwrite a later verdict.
  const checkId = useRef(0);

  const checkUrl = async (url: string) => {
    const id = ++checkId.current;
    if (!url.trim()) {
      setStatus({ tone: "muted", text: "" });
      return null;
    }
    setChecking(true);
    try {
      const d = await gql(
        `query($url:String!){checkRadioStream(url:$url){ok error name genre bitrate codec}}`,
        { url }
      );
      const check = d.checkRadioStream;
      if (id !== checkId.current) return null;
      if (!check.ok) {
        setStatus({ tone: "error", text: check.error });
        return check;
      }
      setStatus({
        tone: "ok",
        text: [
          "Stream reachable",
          check.codec,
          check.bitrate ? `${check.bitrate} kbps` : "",
        ]
          .filter(Boolean)
          .join(" · "),
      });
      // Only fill in what the user has not typed themselves.
      setName((current) => current || check.name);
      setGenre((current) => current || check.genre);
      return check;
    } catch (e) {
      if (id !== checkId.current) return null;
      setStatus({
        tone: "error",
        text: e instanceof Error ? e.message : "Could not check the stream",
      });
      return null;
    } finally {
      if (id === checkId.current) setChecking(false);
    }
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    try {
      const d = await gql(
        `mutation($station:NewRadioStationInput!){addRadioStation(station:$station){${fields}}}`,
        { station: { name, streamUrl, genre, country, logo } }
      );
      onAdded(d.addRadioStation);
      onClose();
    } catch (err) {
      setStatus({
        tone: "error",
        text: err instanceof Error ? err.message : "Could not add the station",
      });
    } finally {
      setSaving(false);
    }
  };

  return (
    <FormPanel onClick={(e) => e.stopPropagation()}>
      <FormTitle>Add a station</FormTitle>
      <FormHint>
        Signed in to atradio.fm, it is published to your account and follows you
        to your other devices.
      </FormHint>
      <form onSubmit={submit}>
        <Field>
          <FieldLabel>Stream url</FieldLabel>
          <FieldInput
            autoFocus
            required
            placeholder="https://example.com/stream"
            value={streamUrl}
            onChange={(e) => {
              setStreamUrl(e.target.value);
              setStatus({ tone: "muted", text: "" });
            }}
            onBlur={(e) => checkUrl(e.target.value)}
          />
        </Field>
        <Status tone={checking ? "muted" : status.tone}>
          {checking ? "Checking the stream…" : status.text}
        </Status>
        <Field>
          <FieldLabel>Name</FieldLabel>
          <FieldInput
            required
            placeholder="Station name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </Field>
        <Field>
          <FieldLabel>Genre</FieldLabel>
          <FieldInput
            placeholder="Optional"
            value={genre}
            onChange={(e) => setGenre(e.target.value)}
          />
        </Field>
        <Field>
          <FieldLabel>Country</FieldLabel>
          <FieldInput
            placeholder="Optional"
            value={country}
            onChange={(e) => setCountry(e.target.value)}
          />
        </Field>
        <Field>
          <FieldLabel>Logo url</FieldLabel>
          <FieldInput
            placeholder="Optional"
            value={logo}
            onChange={(e) => setLogo(e.target.value)}
          />
        </Field>
        <FormActions>
          <Button type="button" onClick={onClose}>
            Cancel
          </Button>
          <Button
            primary
            type="submit"
            disabled={saving || checking || !name.trim() || !streamUrl.trim()}
          >
            {saving ? "Adding…" : "Add station"}
          </Button>
        </FormActions>
      </form>
    </FormPanel>
  );
};

export default function RadioPage() {
  const theme = useTheme();
  const [searchParams, setSearchParams] = useSearchParams();
  const [tab, setTab] = useState<"search" | "saved" | "stations">("search");
  const [stations, setStations] = useState<Station[]>([]);
  const [saved, setSaved] = useState<Station[]>([]);
  const [mine, setMine] = useState<Station[]>([]);
  const [savedLoading, setSavedLoading] = useState(true);
  const [mineLoading, setMineLoading] = useState(true);
  const [addOpen, setAddOpen] = useState(false);
  const [search, setSearch] = useState(false);
  const [q, setQ] = useState("");
  const [category, setCategory] = useState<{ label: string; term: string }>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const requestId = useRef(0);

  const loadSaved = async () => {
    setSavedLoading(true);
    try {
      const d = await gql(`query { savedRadios { ${fields} } }`);
      setSaved(d.savedRadios || []);
    } finally {
      setSavedLoading(false);
    }
  };

  const loadMine = async () => {
    setMineLoading(true);
    try {
      const d = await gql(`query { radioStations { ${fields} } }`);
      setMine(d.radioStations || []);
    } finally {
      setMineLoading(false);
    }
  };

  useEffect(() => {
    loadSaved();
    loadMine();
  }, []);

  useEffect(() => {
    if (searchParams.get("search") === "1") setSearch(true);
  }, [searchParams]);

  const discover = async (category?: string, query?: string) => {
    const id = ++requestId.current;
    setLoading(true);
    setError("");
    try {
      const d = await gql(
        `query($query:String,$category:String){ radios(query:$query,category:$category){${fields}}}`,
        { query, category }
      );
      if (id === requestId.current) setStations(d.radios || []);
    } catch (e) {
      if (id === requestId.current) {
        setStations([]);
        setError(e instanceof Error ? e.message : "Unable to load radio stations");
      }
    } finally {
      if (id === requestId.current) setLoading(false);
    }
  };

  const openCategory = (item: { label: string; term: string }) => {
    setCategory(item);
    setStations([]);
    discover(item.term);
  };

  const play = (s: Station) =>
    gql(`mutation($station:RadioStationInput!){playRadio(station:$station)}`, {
      station: s,
    });

  const toggle = async (s: Station) => {
    const exists = saved.some((x) => x.id === s.id);
    await gql(
      exists
        ? `mutation($id:String!){removeSavedRadio(id:$id)}`
        : `mutation($station:RadioStationInput!){saveRadio(station:$station)}`,
      exists ? { id: s.id } : { station: s }
    );
    // Unbookmarking one of the user's own stations drops it from that list too
    // — locally the two are the same row.
    await Promise.all([loadSaved(), loadMine()]);
  };

  const BookmarkButton: FC<{ station: Station }> = ({ station }) => {
    const bookmarked = saved.some((x) => x.id === station.id);
    return (
      <Action
        onClick={() => toggle(station)}
        title={bookmarked ? "Remove bookmark" : "Bookmark station"}
      >
        {bookmarked ? (
          <Heart height={24} width={24} color="#fe099c" />
        ) : (
          <HeartOutline height={24} width={24} color={theme.colors.icon} />
        )}
      </Action>
    );
  };

  const stationRows = (items: Station[]) =>
    items.map((s) => (
      <Row key={s.id}>
        <RadioArt logo={s.logo} size={44} radius={6} />
        <Info>
          <Name title={s.name}>{s.name}</Name>
          <Meta>{[s.genre, s.country, s.source].filter(Boolean).join(" · ")}</Meta>
        </Info>
        <BookmarkButton station={s} />
        <Action onClick={() => play(s)} title="Play">
          ▶
        </Action>
      </Row>
    ));

  const closeSearch = () => {
    setSearch(false);
    setTab("search");
    if (searchParams.has("search")) setSearchParams({}, { replace: true });
  };

  const tabOverrides = {
    Tab: { style: { backgroundColor: "transparent", borderRadius: 0 } },
    TabPanel: { style: { paddingLeft: 0, paddingRight: 0 } },
  };

  return (
    <Shell>
      <Sidebar active="radio" />
      <Main>
        <ControlBar />
        <Body>
          <PageTitle>Internet Radio</PageTitle>
          {category ? (
            <>
              <CategoryHeader>
                <BackButton
                  onClick={() => {
                    setCategory(undefined);
                    setStations([]);
                  }}
                >
                  ‹
                </BackButton>
                <CategoryTitle>{category.label}</CategoryTitle>
              </CategoryHeader>
              {loading ? (
                <StationLoader />
              ) : error ? (
                <ErrorText>{error}</ErrorText>
              ) : stations.length ? (
                stationRows(stations)
              ) : (
                <EmptyText>No station found in this category.</EmptyText>
              )}
            </>
          ) : (
            <Tabs
              activeKey={tab}
              onChange={({ activeKey }) =>
                setTab(activeKey as "search" | "saved" | "stations")
              }
              overrides={{
                TabList: { style: { marginLeft: 0, marginRight: 0 } },
                TabBorder: { style: { marginLeft: 0, marginRight: 0 } },
                TabHighlight: {
                  style: { height: "2px", backgroundColor: "#ab28fc" },
                },
              }}
            >
              <Tab key="search" title="Search" overrides={tabOverrides}>
                <SearchLauncher onClick={() => setSearch(true)}>
                  <SearchIcon size={21} />
                  <span>Search radio stations…</span>
                </SearchLauncher>
                <Grid>
                  {categories.map((c) => (
                    <Category key={c.label} onClick={() => openCategory(c)}>
                      <CategoryIcon color={c.color}>
                        <c.Icon size={19} />
                      </CategoryIcon>
                      <span>{c.label}</span>
                    </Category>
                  ))}
                </Grid>
              </Tab>
              <Tab key="saved" title="Bookmarked" overrides={tabOverrides}>
                {savedLoading ? (
                  <StationLoader />
                ) : saved.length ? (
                  stationRows(saved)
                ) : (
                  <EmptyText>No bookmarked station yet.</EmptyText>
                )}
              </Tab>
              <Tab key="stations" title="Stations" overrides={tabOverrides}>
                <AddStationButton onClick={() => setAddOpen(true)}>
                  <span>+</span>
                  <span>Add a station</span>
                </AddStationButton>
                {mineLoading ? (
                  <StationLoader />
                ) : mine.length ? (
                  stationRows(mine)
                ) : (
                  <EmptyText>
                    No station of your own yet. Add one with its stream url.
                  </EmptyText>
                )}
              </Tab>
            </Tabs>
          )}
        </Body>
      </Main>
      {addOpen && (
        <Modal onClick={() => setAddOpen(false)}>
          <AddStationForm
            onClose={() => setAddOpen(false)}
            onAdded={() => {
              loadSaved();
              loadMine();
            }}
          />
        </Modal>
      )}
      {search && (
        <Modal onClick={closeSearch}>
          <Panel onClick={(e) => e.stopPropagation()}>
            <SearchField>
              <SearchIcon size={21} />
              <Input
                autoFocus
                placeholder="Search radio stations…"
                value={q}
                onChange={(e) => {
                  const value = e.target.value;
                  setQ(value);
                  if (value.trim()) {
                    discover(undefined, value);
                  } else {
                    requestId.current++;
                    setStations([]);
                    setLoading(false);
                  }
                }}
              />
            </SearchField>
            {loading ? (
              <StationLoader rows={5} />
            ) : error ? (
              <ErrorText>{error}</ErrorText>
            ) : (
              stations.map((s) => (
                <Row key={s.id}>
                  <RadioArt logo={s.logo} size={44} radius={6} />
                  <Info>
                    <Name title={s.name}>{s.name}</Name>
                    <Meta>
                      {[s.genre, s.country, s.source].filter(Boolean).join(" · ")}
                    </Meta>
                  </Info>
                  <BookmarkButton station={s} />
                  <Action
                    onClick={() => {
                      play(s);
                      closeSearch();
                    }}
                    title="Play"
                  >
                    ▶
                  </Action>
                </Row>
              ))
            )}
          </Panel>
        </Modal>
      )}
    </Shell>
  );
}
