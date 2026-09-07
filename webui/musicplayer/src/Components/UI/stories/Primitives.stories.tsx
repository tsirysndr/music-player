import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { noop } from "../../../Stories/fixtures";
import {
  AlbumSkeletonGrid,
  Artwork,
  Button,
  EmptyState,
  FilterBox,
  IconButton,
  Icons,
  LoadMore,
  MarqueeText,
  PageToolbar,
  RadioSkeletonList,
  SearchButton,
  SectionHeader,
  SidebarItem,
  SkeletonBox,
  TrackSkeletonList,
} from "..";

const COVER =
  "https://resources.tidal.com/images/f87d9afc/075e/43f4/bbbc/7770b46cb8aa/320x320.jpg";

/** The small parts everything else is assembled from. */
const meta: Meta = {
  title: "UI/Primitives",
  decorators: [
    (Story) => (
      <div className="w-full bg-window p-6">
        <Story />
      </div>
    ),
  ],
};

export default meta;

type Story = StoryObj;

/**
 * The art box, with the skin's placeholder behind it. The fallback glyph is
 * sized as a fraction of the box, so a 60px thumbnail and a 200px card both
 * read at a glance.
 */
export const ArtworkSizes: Story = {
  name: "Artwork",
  render: () => (
    <div className="flex flex-wrap items-end gap-5">
      <Artwork src={COVER} alt="Discovery" className="size-[52px]" />
      <Artwork src={COVER} alt="Discovery" className="size-[150px]" />
      <Artwork alt="No cover" className="size-[52px]" />
      <Artwork alt="No cover" className="size-[150px]" />
      <Artwork
        fallbackIcon={Icons.artist}
        rounded="full"
        alt="No portrait"
        className="size-[150px]"
      />
      <Artwork
        fallbackIcon={Icons.broadcast}
        alt="Station"
        className="size-[150px] [&_svg]:text-accent"
      />
    </div>
  ),
};

export const Skeletons: Story = {
  render: () => (
    <div className="flex flex-col gap-8">
      <div>
        <SectionHeader title="BOXES" className="mb-3" />
        <div className="flex items-center gap-3">
          <SkeletonBox className="size-[52px]" />
          <div className="flex flex-1 flex-col gap-2">
            <SkeletonBox className="h-3 w-1/3" />
            <SkeletonBox className="h-3 w-1/4" />
          </div>
        </div>
      </div>
      <div>
        <SectionHeader title="TRACK LIST" className="mb-3" />
        <TrackSkeletonList rows={5} />
      </div>
      <div>
        <SectionHeader title="ALBUM GRID" className="mb-3" />
        <AlbumSkeletonGrid />
      </div>
      <div>
        <SectionHeader title="RADIO LIST" className="mb-3" />
        <RadioSkeletonList rows={4} />
      </div>
    </div>
  ),
};

export const EmptyStates: Story = {
  render: () => (
    <div className="flex flex-col gap-10">
      <EmptyState
        icon={Icons.disc}
        title="Nothing here yet"
        hint="Scan your library with `music-player scan`."
      />
      <EmptyState
        icon={Icons.playlist}
        title="No playlists"
        hint="Group the tracks you keep coming back to."
        action={<Button onClick={noop}>New playlist</Button>}
      />
      <EmptyState title="No results" />
    </div>
  ),
};

export const SectionHeaders: Story = {
  render: () => (
    <div className="flex flex-col gap-4">
      <SectionHeader title="ALBUMS" />
      <SectionHeader title="SONGS" />
      <SectionHeader title="EQUALIZER" />
    </div>
  ),
};

export const SidebarItems: Story = {
  render: () => (
    <div className="flex w-[184px] flex-col gap-1 bg-sidebar p-2">
      <SidebarItem icon={Icons.disc} label="Albums" to="/albums" />
      <SidebarItem icon={Icons.artist} label="Artists" to="/artists" active />
      <SidebarItem icon={Icons.listMusic} label="Tracks" to="/tracks" />
      <SidebarItem icon={Icons.server} label="Servers" to="/servers" />
      <SidebarItem
        icon={Icons.playlist}
        label="A playlist with a very long name"
        to="/playlists/1"
      />
      <SidebarItem icon={Icons.settings} label="As a button" onClick={noop} />
    </div>
  ),
};

/** The header's search affordance — it opens the palette, it is not a field. */
export const SearchButtons: Story = {
  render: () => (
    <div className="flex flex-col gap-3">
      <SearchButton onClick={noop} />
      <SearchButton onClick={noop} className="w-[320px]" />
    </div>
  ),
};

export const FilterBoxes: Story = {
  render: () => {
    const [value, setValue] = useState("");
    return (
      <div className="flex flex-col gap-3">
        <FilterBox value={value} onChange={setValue} />
        <FilterBox
          value="black sabbath"
          placeholder="Filter tracks"
          onChange={noop}
        />
      </div>
    );
  },
};

/**
 * The strip above a list. Every page's filter lives here rather than in the
 * header — the header is for the global search only.
 */
export const PageToolbars: Story = {
  render: () => (
    <div className="flex flex-col gap-6">
      <PageToolbar filter="" filterPlaceholder="Filter albums" onFilter={noop}>
        <span className="text-xs text-muted">248 albums</span>
      </PageToolbar>
      <PageToolbar
        trailing={
          <IconButton icon={Icons.refresh} iconSize={15} aria-label="Rescan" />
        }
      >
        <span className="text-xs text-muted">12 extensions</span>
      </PageToolbar>
      <PageToolbar>
        <span className="text-xs text-muted">Nothing but a caption</span>
      </PageToolbar>
    </div>
  ),
};

export const LoadMoreStates: Story = {
  name: "LoadMore",
  render: () => (
    <div className="flex flex-col gap-6">
      <LoadMore hasMore onLoadMore={noop} />
      {/* Nothing left: renders nothing, so the list just ends. */}
      <LoadMore onLoadMore={noop} />
    </div>
  ),
};

/**
 * Scrolls itself when the text overruns its box, and sits still when it fits —
 * the player bar's title and the ICY `StreamTitle` both rely on it.
 */
export const MarqueeTexts: Story = {
  name: "MarqueeText",
  render: () => (
    <div className="flex w-[220px] flex-col gap-4 bg-panel p-3">
      <MarqueeText text="Otherside" className="text-[13px] text-fg" />
      <MarqueeText
        text="A title long enough that it has to scroll rather than be cut off"
        className="text-[13px] font-semibold text-fg"
      />
      <MarqueeText
        text="Red Hot Chili Peppers · Californication · 1999"
        className="text-[11px] text-dim"
      />
    </div>
  ),
};
