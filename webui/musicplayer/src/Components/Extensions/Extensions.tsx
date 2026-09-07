import { FC } from "react";
import { AppShell } from "../Layout";
import {
  EmptyState,
  IconButton,
  Icons,
  PageToolbar,
  SkeletonBox,
  Toggle,
  cn,
} from "../UI";

/** The quick filter above the list. */
export type StatusFilter = "all" | "enabled" | "disabled";

const STATUS_FILTERS: { value: StatusFilter; label: string }[] = [
  { value: "all", label: "All" },
  { value: "enabled", label: "Enabled" },
  { value: "disabled", label: "Disabled" },
];

export type ExtensionItem = {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  homepage: string;
  repository: string;
  license: string;
  logo: string;
  topics: string[];
  capabilities: string[];
  allowedHosts: string[];
  libraryRead: boolean;
  /** `enabled` or `disabled`. */
  status: string;
  path: string;
};

export type ExtensionsProps = {
  extensions: ExtensionItem[];
  loading?: boolean;
  error?: string;
  status: StatusFilter;
  /** Ids with a toggle in flight, so their switch is held until it lands. */
  pending?: string[];
  onStatusFilter: (status: StatusFilter) => void;
  onToggle: (id: string, enabled: boolean) => void;
  onRescan: () => void;
};

/**
 * What each capability means, in one line. The manifest declares them as bare
 * strings; a list of five nouns tells a user nothing about what an extension
 * is allowed to do, which is the thing they are here to check.
 */
const CAPABILITY_HINTS: Record<string, string> = {
  events: "Reacts to plays, skips, likes and scans",
  metadata: "Supplies lyrics, artwork, bios or genres",
  commands: "Adds actions to the UI",
  predicates: "Adds smart-playlist filter terms",
  source: "Provides a browsable media source",
};

const CapabilityChip = ({ capability }: { capability: string }) => (
  <span
    title={CAPABILITY_HINTS[capability]}
    className="rounded-full border border-line bg-hover px-2 py-[2px] font-mono text-[10px] text-dim"
  >
    {capability}
  </span>
);

const ExtensionCard = ({
  extension,
  pending,
  onToggle,
}: {
  extension: ExtensionItem;
  pending?: boolean;
  onToggle: (id: string, enabled: boolean) => void;
}) => {
  const enabled = extension.status === "enabled";
  const link = extension.homepage || extension.repository;

  return (
    <li className="flex flex-col gap-3 rounded-skin bg-panel p-4">
      <div className="flex items-start gap-3">
        <span className="grid size-10 shrink-0 place-items-center overflow-hidden rounded-control bg-art">
          {extension.logo ? (
            <img
              src={extension.logo}
              alt=""
              className="size-full object-cover"
            />
          ) : (
            <Icons.extension size={20} className="text-accent" />
          )}
        </span>

        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-baseline gap-x-2">
            <h3 className="truncate text-[15px] font-semibold text-fg">
              {extension.name}
            </h3>
            {extension.version && (
              <span className="font-mono text-[11px] text-muted">
                v{extension.version}
              </span>
            )}
          </div>
          <p className="truncate font-mono text-[11px] text-muted">
            {extension.id}
          </p>
        </div>

        <div className="flex shrink-0 items-center gap-[10px]">
          <span
            className={cn(
              "text-[10px] font-semibold",
              enabled ? "text-accent" : "text-muted"
            )}
          >
            {enabled ? "Enabled" : "Disabled"}
          </span>
          <Toggle
            checked={enabled}
            disabled={pending}
            label={`Enable ${extension.name}`}
            onChange={(next) => onToggle(extension.id, next)}
          />
        </div>
      </div>

      {extension.description && (
        <p className="text-[13px] text-dim">{extension.description}</p>
      )}

      <div className="flex flex-wrap gap-[6px]">
        {extension.capabilities.map((capability) => (
          <CapabilityChip key={capability} capability={capability} />
        ))}
        {extension.topics.map((topic) => (
          <span
            key={topic}
            className="rounded-full px-2 py-[2px] text-[10px] text-muted"
          >
            #{topic}
          </span>
        ))}
      </div>

      {/* What it may reach outside its sandbox. Worth stating plainly: a
          WebAssembly module gets nothing by default, and these are the doors
          its manifest asked to open. */}
      {(extension.allowedHosts.length > 0 || extension.libraryRead) && (
        <div className="flex flex-col gap-1 border-t border-line pt-3 text-[11px] text-muted">
          {extension.libraryRead && (
            <span className="flex items-center gap-2">
              <Icons.listMusic size={12} />
              Reads your library
            </span>
          )}
          {extension.allowedHosts.length > 0 && (
            <span className="flex items-start gap-2">
              <Icons.broadcast size={12} className="mt-[2px] shrink-0" />
              <span className="font-mono">
                {extension.allowedHosts.join(", ")}
              </span>
            </span>
          )}
        </div>
      )}

      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted">
        {extension.author && <span>by {extension.author}</span>}
        {extension.license && <span>{extension.license}</span>}
        {link && (
          <a
            href={link}
            target="_blank"
            rel="noreferrer noopener"
            className="text-accent hover:underline"
          >
            Website
          </a>
        )}
      </div>
    </li>
  );
};

/**
 * The installed extensions — the web client's half of the same view the
 * desktop client shows in its own Extensions tab.
 */
const Extensions: FC<ExtensionsProps> = ({
  extensions,
  loading,
  error,
  status,
  pending = [],
  onStatusFilter,
  onToggle,
  onRescan,
}) => (
  <AppShell>
    {/* No search here: extensions are reachable from the command palette
        (`/` or ⌘K), which searches them alongside everything else. */}
    <PageToolbar
      trailing={
        <IconButton
          icon={Icons.refresh}
          iconSize={16}
          aria-label="Rescan extensions"
          onClick={onRescan}
        />
      }
    >
      <div className="flex gap-[6px]">
        {STATUS_FILTERS.map((entry) => (
          <button
            key={entry.value}
            type="button"
            aria-pressed={status === entry.value}
            onClick={() => onStatusFilter(entry.value)}
            className={cn(
              "h-[26px] rounded-full px-3 text-[11px] transition-colors",
              status === entry.value
                ? "border border-accent/40 bg-selected font-semibold text-accent"
                : "text-dim hover:bg-hover"
            )}
          >
            {entry.label}
          </button>
        ))}
      </div>
    </PageToolbar>

    {loading ? (
      <ul className="grid gap-3 lg:grid-cols-2 2xl:grid-cols-3">
        {Array.from({ length: 4 }, (_, i) => (
          <li key={i} className="rounded-skin bg-panel p-4">
            <div className="flex gap-3">
              <SkeletonBox className="size-10 rounded-control" />
              <div className="flex flex-1 flex-col gap-2">
                <SkeletonBox className="h-4 w-1/3" />
                <SkeletonBox className="h-3 w-1/2" />
              </div>
            </div>
            <SkeletonBox className="mt-4 h-3 w-full" />
          </li>
        ))}
      </ul>
    ) : error ? (
      <EmptyState
        icon={Icons.extension}
        title="Unable to load extensions"
        hint={error}
      />
    ) : extensions.length === 0 ? (
      <EmptyState
        icon={Icons.extension}
        title={
          status !== "all"
            ? `No ${status} extensions`
            : "No extensions installed"
        }
        hint={
          status !== "all"
            ? undefined
            : "Create one with `music-player extension init <id>`, or install one with `music-player extension install <url>`."
        }
      />
    ) : (
      <ul className="grid gap-3 lg:grid-cols-2 2xl:grid-cols-3">
        {extensions.map((extension) => (
          <ExtensionCard
            key={extension.id}
            extension={extension}
            pending={pending.includes(extension.id)}
            onToggle={onToggle}
          />
        ))}
      </ul>
    )}
  </AppShell>
);

export default Extensions;
