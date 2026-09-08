import { useTimeFormat } from "../../Hooks/useFormat";
import { resourceUriResolver } from "../../ResourceUriResolver";
import {
  Artwork,
  IconButton,
  Icons,
  Knob,
  LikeButton,
  MarqueeText,
  PlayPauseButton,
  SlideBar,
  VfdDisplay,
  cn,
} from "../UI";

export type PlayerBarProps = {
  title?: string;
  artist?: string;
  /** The album, or the station name while a stream is playing. */
  album?: string;
  cover?: string | null;
  /** Nothing is loaded: the transport is inert and the VFD shows a square. */
  stopped?: boolean;
  playing?: boolean;
  /** Milliseconds. */
  progress?: number;
  duration?: number;
  /** A live stream: no seek, no skip, and the heart is a bookmark. */
  isRadio?: boolean;
  /** Liked, or — for a station — bookmarked. */
  liked?: boolean;
  /** 0..1 */
  volume?: number;
  /** Silenced at the daemon, which keeps `volume` so unmuting restores it. */
  muted?: boolean;
  queueOpen?: boolean;
  audioSettingsOpen?: boolean;
  /**
   * The full player is open above it. The bar goes translucent over a blurred
   * copy of the art, which is what the desktop does — the canvas above stops
   * short of the bar rather than covering it, so the transport stays here.
   */
  overlay?: boolean;

  onPlay: () => void;
  onPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  /** Milliseconds. */
  onSeek: (positionMs: number) => void;
  onToggleLike: () => void;
  onVolume: (volume: number) => void;
  onToggleMute: () => void;
  onToggleQueue: () => void;
  onOpenFullPlayer: () => void;
  onOpenAudioSettings: () => void;
  /** Opens the "play on" picker — where the audio comes out. */
  onOpenDevices: () => void;
  /** A cast target is playing, rather than this machine. */
  castingTo?: string;
  onShuffle?: () => void;
  onRepeat?: () => void;
};

/**
 * The desktop's player bar: now-playing at the left, transport and seek in the
 * middle, and the VFD readout plus the volume knob at the right.
 *
 * Below `lg` the VFD and the knob are dropped and the whole bar becomes a
 * single tappable strip over the bottom tabs — a 220px readout and a rotary
 * are not usable on a phone, and the seek slider is what people reach for.
 *
 * Presentational: `PlayerBarWithData` supplies the state. That split is what
 * lets every state of it be seen at once in Storybook, including the ones that
 * are awkward to reach in the running app.
 */
const PlayerBar = ({
  title,
  artist,
  album,
  cover,
  stopped = !title,
  playing,
  progress = 0,
  duration = 0,
  isRadio,
  liked,
  volume = 1,
  muted,
  queueOpen,
  audioSettingsOpen,
  overlay,
  onPlay,
  onPause,
  onNext,
  onPrevious,
  onSeek,
  onToggleLike,
  onVolume,
  onToggleMute,
  onToggleQueue,
  onOpenFullPlayer,
  onOpenAudioSettings,
  onOpenDevices,
  castingTo,
  onShuffle,
  onRepeat,
}: PlayerBarProps) => {
  const { formatTime } = useTimeFormat();
  const position = duration > 0 ? progress / duration : 0;

  /**
   * The second line. For a station the album slot holds the station name, so
   * once ICY metadata names the song on air this reads "Artist · Station";
   * before that the title already *is* the station name, so drop the repeat.
   */
  const showStation =
    isRadio && !!album && album !== artist && album !== title;
  const subtitle = !showStation
    ? (artist ?? "")
    : [artist, album].filter(Boolean).join(" · ");

  const blurred = overlay
    ? resourceUriResolver.resolve(cover ?? undefined)
    : undefined;

  return (
    <div
      className={cn(
        // The height matches `--player-bar-height` in `index.css`, which is
        // what the full-player canvas stops short of.
        "relative z-50 flex h-[76px] shrink-0 items-center gap-3 border-t border-line px-3",
        "lg:h-[92px] lg:gap-[18px] lg:px-4",
        overlay ? "border-white/10 bg-[#100c18]/[0.87]" : "bg-panel"
      )}
    >
      {blurred && (
        <>
          <div
            aria-hidden="true"
            style={{ backgroundImage: `url("${blurred}")` }}
            className="absolute -inset-5 -z-10 bg-cover bg-center opacity-[0.28] blur-2xl"
          />
          <div aria-hidden="true" className="absolute inset-0 -z-10 bg-[#100c18]/[0.67]" />
        </>
      )}
      {/* Now playing */}
      <div className="flex min-w-0 flex-1 items-center gap-3 lg:w-[26%] lg:flex-none">
        {/* Hidden while the full player is open: the art is already the whole
            canvas above, and a thumbnail of it here is just a second copy —
            the desktop hides it for the same reason. */}
        {!overlay && (cover || isRadio) && (
          <button
            type="button"
            // The `f` shortcut does the same thing; naming it here is how a
            // user finds out it exists.
            aria-label="Open full player"
            title="Open full player (f)"
            disabled={stopped}
            onClick={onOpenFullPlayer}
            className="shrink-0"
          >
            <Artwork
              src={cover}
              alt={title ?? ""}
              fallbackIcon={isRadio ? Icons.broadcast : Icons.music}
              iconSize={24}
              className="size-[52px] lg:size-[60px] [&_svg]:text-accent"
            />
          </button>
        )}
        <div className="flex min-w-0 flex-1 flex-col justify-center gap-[3px]">
          <MarqueeText
            text={title ?? "Nothing playing"}
            className="text-[13px] font-semibold text-fg"
          />
          <MarqueeText text={subtitle} className="text-[11px] text-dim" />
        </div>
        {!stopped && (
          <LikeButton
            liked={liked}
            iconSize={21}
            size={32}
            // Nudged onto the title line rather than centred between title and
            // subtitle — the desktop offsets its heart by the same 8px.
            className="hidden -translate-y-2 sm:grid"
            onClick={onToggleLike}
          />
        )}
      </div>

      {/* Transport + seek */}
      <div className="flex flex-col items-center justify-center gap-[6px] lg:flex-1">
        <div className="flex items-center gap-[10px]">
          {!isRadio && (
            <IconButton
              icon={Icons.shuffle}
              iconSize={15}
              aria-label="Shuffle"
              className="hidden sm:inline-flex"
              onClick={() => onShuffle?.()}
            />
          )}
          {!isRadio && (
            <IconButton
              icon={Icons.prev}
              iconSize={17}
              aria-label="Previous"
              onClick={onPrevious}
            />
          )}
          <PlayPauseButton
            playing={playing}
            size={40}
            className="lg:size-11"
            onClick={playing ? onPause : onPlay}
          />
          {!isRadio && (
            <IconButton
              icon={Icons.next}
              iconSize={17}
              aria-label="Next"
              onClick={onNext}
            />
          )}
          {!isRadio && (
            <IconButton
              icon={Icons.repeat}
              iconSize={15}
              aria-label="Repeat"
              className="hidden sm:inline-flex"
              onClick={() => onRepeat?.()}
            />
          )}
        </div>
        {!isRadio && (
          <div className="hidden w-full items-center gap-[10px] lg:flex">
            <span className="font-mono text-[10px] text-muted tabular-nums">
              {formatTime(progress)}
            </span>
            <SlideBar
              progress={position}
              disabled={stopped}
              onChange={(value) => onSeek(Math.round(value * duration))}
            />
            <span className="font-mono text-[10px] text-muted tabular-nums">
              {formatTime(duration)}
            </span>
          </div>
        )}
      </div>

      {/* VFD + volume */}
      <div className="hidden items-center gap-[10px] lg:flex">
        {/* Where the audio comes out. The sidebar's status row is the other
            question — where the library is read *from*. */}
        <IconButton
          icon={Icons.cast}
          iconSize={17}
          accented={!!castingTo}
          aria-label={castingTo ? `Playing on ${castingTo}` : "Play to"}
          title={castingTo ? `Playing on ${castingTo}` : "Play to"}
          onClick={onOpenDevices}
        />
        <IconButton
          icon={Icons.equalizer}
          iconSize={16}
          accented={audioSettingsOpen}
          aria-label="Audio settings"
          onClick={onOpenAudioSettings}
        />
        <VfdDisplay
          // A live stream has no meaningful position, so the readout names
          // the source instead of counting.
          timeText={isRadio ? "RADIO" : formatTime(progress)}
          infoText={isRadio ? "internet radio" : (album ?? "—")}
          playing={playing}
          stopped={stopped}
          vuLeft={playing ? 0.55 : 0}
          vuRight={playing ? 0.5 : 0}
          // Shown wherever the bar is in its desktop layout, as on the desktop
          // client. It narrows between lg and xl so the transport keeps room.
          className="w-[168px] xl:w-[220px]"
        />
        {/* Mute is a button as well as `m`: a shortcut nobody can see is a
            shortcut nobody uses. */}
        <IconButton
          icon={muted ? Icons.volumeMute : Icons.volume}
          iconSize={17}
          accented={muted}
          aria-label={muted ? "Unmute" : "Mute"}
          aria-pressed={!!muted}
          onClick={onToggleMute}
        />
        <Knob
          size={52}
          gap={2}
          // Muted keeps the level, so the readout says so rather than showing
          // a number you cannot hear.
          valueText={muted ? "muted" : `${Math.round(volume * 100)}%`}
          norm={volume}
          defaultNorm={0.75}
          aria-label="Volume"
          onChange={onVolume}
        />
      </div>

      {/* On a phone only one fits, and the queue has nowhere else to live
          there — the right panel is a desktop layout. */}
      <IconButton
        icon={Icons.listMusic}
        iconSize={18}
        accented={queueOpen}
        aria-label="Play queue"
        className="lg:hidden"
        onClick={onToggleQueue}
      />
    </div>
  );
};

export default PlayerBar;
