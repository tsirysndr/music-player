---
name: music-player
description: Play, queue and search music on the user's music-player daemon, and DJ for them — building a set that fits a mood, an activity or a request. Use whenever the user asks to play or queue something, asks what is playing, wants the volume or transport changed, or asks for a set, a mix or a playlist to be put on.
---

# Playing music

You are driving a real audio player on the user's machine. Everything you do is
heard immediately, by whoever is in the room. That is the whole reason this is
worth being careful about: a wrong `play_tracks` does not produce a wrong
answer, it cuts off the song someone was enjoying.

## Before anything else

Check that the tools are there. If `now_playing` and the rest are not available,
the MCP server is not connected — see [Setting it up](#setting-it-up) and tell
the user what to run. Do not fall back to guessing.

If the tools are there but every call reports no daemon at `host:port`, the CLI
is installed but nothing is running. Tell the user to start it:

```bash
music-player          # foreground; add & to keep the shell
```

## How to work

**Find, then play.** Ids are the currency. `search` takes free text and hands
back track, album and artist ids; `browse_library` pages through everything when
there is nothing specific to look for. Both feed `play_tracks` and
`queue_tracks`.

**Queue rather than replace.** `queue_tracks` adds without touching what is
playing. `play_tracks` wipes the queue and starts over — right when the user
says "play X", wrong when they say "add some more like this". When in doubt,
queue: an unwanted queue entry is skipped, an unwanted interruption is not
recoverable.

**Look before you change.** Call `now_playing` first when the request depends on
context — "something like this", "more of the same", "after this one". It also
tells you whether anything is playing at all, which changes whether you should
queue or start.

**Say what you did.** "Queued 8 tracks" is not an answer. Name a few of them —
the user cannot see your tool calls, and the point of asking an agent rather
than clicking is that you can say *why* those tracks.

## Knowing what a track sounds like

`search` and `browse_library` only know titles. `track_analysis` knows how a
track actually sounds — its tempo, its energy (0 calm to 1 driving), its
brightness (-1 dark to 1 bright), and how loud it is. `similar_tracks` uses all
of that to answer "what goes after this?" without needing genres or anyone
else's listening history.

None of it works until tracks have been analysed, because analysis means
decoding the audio. `analyze_library` starts a background pass and returns
immediately; call it with no arguments to see how far it has got. Run it once
for a library, and check it before promising a mood-based set:

- `analyze_library` with no arguments → `{"analyzed": 0, "running": false}`
  means nothing is analysed and `similar_tracks` will fail. Say so, and offer
  to start a pass rather than silently falling back to searching by name.
- A pass takes roughly a second per local track and longer for a remote server,
  since each one has to be fetched. Start it, tell the user it is running, and
  get on with building a set from what is already there.

## Handing over to auto-DJ

`auto_dj` turns the daemon's own DJ on: it keeps the queue five tracks deep,
each one chosen to follow the last by tempo and mood, never repeating and never
playing the same artist twice in a row. It never interrupts what is playing.

Steer it rather than micromanaging it. `energy`, `brightness` and `bpm` are a
*direction*, not a filter — the set leans that way over the next few tracks
instead of jumping:

- "wind things down" → `auto_dj` with `enabled: true, energy: 0.25`
- "keep it going but pick it up" → `energy: 0.8`
- "something more upbeat and brighter" → `energy: 0.75, brightness: 0.6`
- "just keep playing" → `enabled: true` with no target at all
- "stop choosing for me" → `enabled: false`

Prefer auto-DJ over queueing thirty tracks yourself when the user wants music
to keep going indefinitely: it reacts to skips and to a changed target, which a
queue you built half an hour ago cannot.

## DJing

When asked for a set — "put on something for cooking", "build me an hour of
focus music", "DJ for the party" — the work is selection, not tool calls.

- **Search around the request, not just inside it.** A request for "something
  upbeat" is not a search query. Look for the artists, genres and albums that
  would satisfy it, several searches if needed, then choose from what the
  library actually holds.
- **Sequence deliberately.** Order is the DJ's contribution: open in the mood
  asked for, build, and do not put the two most similar tracks back to back.
  `play_tracks` and `queue_tracks` both play in the order given.
- **Keep it to a set, not a dump.** Ten to twenty well-chosen tracks beats a
  hundred filtered ones. If the user wants it endless, queue a set and offer to
  extend it when it runs low.
- **Work from their library.** You can only play what is there. If the request
  cannot be met — no jazz in the library at all — say so plainly and offer the
  nearest thing, rather than queueing something that merely matched a word.
- **Use the analysis when the request is about feel.** "Something calm", "more
  energy", "something for running" are questions about tempo and energy, not
  about titles. Find one track that fits, then `similar_tracks` from it — that
  is a far better set than guessing from names.

## The tools

| Tool | For |
| --- | --- |
| `now_playing` | Current track, position, volume, how much is queued |
| `playback_control` | `play`, `pause`, `next`, `previous`, `stop` |
| `seek` | Jump to a position in the current track |
| `set_volume` | 0–100 |
| `search` | Tracks, albums and artists matching free text |
| `browse_library` | Page through albums, artists or tracks |
| `get_album` | An album with its tracks, in order |
| `get_artist` | An artist with their albums and tracks |
| `list_playlists` / `get_playlist` | The user's saved playlists |
| `play_tracks` | Replace the queue and start — interrupts |
| `queue_tracks` | Add to the queue, `next` or `end` — does not interrupt |
| `view_queue` | What has played, what is coming |
| `clear_queue` | Empty it and stop |
| `set_playback_mode` | Shuffle on/off, repeat `off`/`queue`/`track` |
| `list_servers` / `connect_server` | Switch which library is being read |
| `track_analysis` | Tempo, energy, brightness, loudness of one track |
| `similar_tracks` | What sounds good after a given track |
| `analyze_library` | Analyse tracks in the background, and check progress |
| `auto_dj` | Hand the queue over to the daemon, and steer it |

`connect_server` changes where music is *browsed from*, not where it comes out;
it never interrupts playback. Only use it if the user asks for a different
server — their library is otherwise whatever is already connected.

## Things to get right

- **Never clear or replace the queue without being asked.** "Play some jazz"
  while something is on is ambiguous: prefer queueing and say that you queued
  rather than replaced, so the user can ask for the other thing.
- **Volume is heard by people, not read by them.** Do not raise it beyond what
  was asked, and do not set it at all unless asked.
- `playback_control` with `play` resumes what is loaded. It starts nothing on an
  empty queue — use `play_tracks` for that.
- A tool that returns `isError` has failed and changed nothing. Read the message
  and tell the user; do not retry the same call.

## Setting it up

Only needed once, and only if the tools are missing.

### 1. Install the CLI

macOS or Linux, prebuilt binary:

```bash
curl -fsSL https://raw.githubusercontent.com/tsirysndr/music-player/master/install.sh | sh
```

Homebrew (macOS/Linux):

```bash
brew install tsirysndr/tap/musicplayer
```

npm:

```bash
npm install -g @tsiry/music-player
```

Debian, Ubuntu and other APT systems:

```bash
echo "deb [trusted=yes] https://apt.fury.io/tsiry/ /" \
  | sudo tee /etc/apt/sources.list.d/music-player.list
sudo apt-get update && sudo apt-get install music-player
```

Fedora, RHEL, Rocky, AlmaLinux and other DNF systems:

```bash
sudo tee /etc/yum.repos.d/music-player.repo <<'EOF'
[music-player]
name=Music Player
baseurl=https://yum.fury.io/tsiry/
enabled=1
gpgcheck=0
EOF
sudo dnf install music-player
```

Check it worked with `music-player --version`.

### 2. Point it at some music

The daemon reads `~/Music` by default; set `music_directory` in
`~/.config/music-player/settings.toml` for anywhere else. Then:

```bash
music-player scan     # index the library
music-player          # start the daemon
```

A remote library — Navidrome, Subsonic, Jellyfin, Plex, Kodi — is added in the
desktop or web UI under Servers, and needs no scan.

### 3. Register the MCP server

Claude Code:

```bash
claude mcp add music-player -- music-player mcp
```

Codex (`~/.codex/config.toml`):

```toml
[mcp_servers.music-player]
command = "music-player"
args = ["mcp"]
```

Claude Desktop, and other hosts that use the same file
(`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS,
`%APPDATA%\Claude\claude_desktop_config.json` on Windows):

```json
{
  "mcpServers": {
    "music-player": {
      "command": "music-player",
      "args": ["mcp"]
    }
  }
}
```

Restart the host afterwards. `music-player mcp` speaks JSON-RPC on stdin and
stdout and is not meant to be run by hand — running it in a terminal looks like
a hang, because it is waiting for a client.
