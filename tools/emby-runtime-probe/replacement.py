"""CI-only public Player replacement probe (Linux, Python standard library).

Build src/bin/replacement.rs in CI, then run this file with --exe PATH.
PCM evidence is TCP delivery, never audible consumption. A 5s controller
watchdog only cleans up failure; cancellation and transition gates are 2s.
"""
import argparse
import array
import collections
import hashlib
import json
import math
import os
import pathlib
import selectors
import socket
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent
RATE, FRAMES, FRAME_BYTES = 44100, 4410, 4
WINDOW_BYTES = FRAMES * FRAME_BYTES


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fixture(path, frequency):
    subprocess.run([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
        "-f", "lavfi", "-i", f"sine=frequency={frequency}:sample_rate={RATE}",
        "-t", "30", "-ac", "2", "-codec:a", "libmp3lame", "-b:a", "128k",
        "-write_xing", "0", "-id3v2_version", "0", str(path),
    ], check=True, timeout=30, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return path.read_bytes()


def spectrum(block):
    samples = array.array("h")
    samples.frombytes(block)
    if sys.byteorder != "little":
        samples.byteswap()
    rms, fractions = [], {440: [], 880: []}
    for channel in (samples[0::2], samples[1::2]):
        energy = sum(float(value) * value for value in channel)
        rms.append(math.sqrt(energy / FRAMES))
        for frequency in fractions:
            coefficient = 2 * math.cos(2 * math.pi * frequency / RATE)
            previous = previous2 = 0.0
            for value in channel:
                current = value + coefficient * previous - previous2
                previous2, previous = previous, current
            power = previous**2 + previous2**2 - coefficient * previous * previous2
            fraction = 2 * max(0.0, power) / (FRAMES * energy) if energy else 0.0
            fractions[frequency].append(min(1.0, fraction))
    tone = "other"
    for label, wanted, unwanted in (("A", 440, 880), ("B", 880, 440)):
        if min(rms) >= 100 and min(fractions[wanted]) >= .8 and max(fractions[unwanted]) <= .01:
            tone = label
    return {"tone": tone, "rms": rms, "fraction_440": fractions[440],
            "fraction_880": fractions[880],
            "nonzero_samples": sum(value != 0 for value in samples)}


def released(session):
    return bool(session) and all(session.get(key) is True for key in
                                ("cancel_requested", "reader_released",
                                 "decoder_joined", "worker_exited"))


class Case:
    def __init__(self, name, exe, media, dest):
        self.name, self.exe, self.media, self.dest = name, exe, media, dest
        self.start = time.monotonic()
        self.result = {"case": name, "pass": False, "errors": [], "commands": [],
                       "engine_events": [], "observer_events": [], "stdout": [],
                       "stderr": [], "parse_errors": [], "requests": [],
                       "pcm_chunks": [], "frequency_windows": [],
                       "watchdog_killed": False, "cleanup_killed": False}
        self.sel = selectors.DefaultSelector()
        self.sockets, self.pipes, self.buffers = {}, {}, {}
        self.proc = self.pcm_file = None
        self.pending = self.latest = self.ready = None
        self.next_poll = 0
        self.next_id = self.pcm_bytes = self.pcm_connections = 0
        self.child_bytes = 0
        self.pcm_pending = bytearray()
        self.segments = collections.deque()
        self.pcm_hash = hashlib.sha256()
        self.finishing = False

    def ms(self):
        return (time.monotonic() - self.start) * 1000

    def stamp(self, event, **detail):
        value = {"ms": self.ms(), "event": event, **detail}
        self.result["observer_events"].append(value)
        return value

    def close_socket(self, sock):
        try:
            try:
                self.sel.unregister(sock)
            except KeyError:
                pass
        finally:
            self.sockets.pop(sock, None)
            sock.close()

    def listen(self, kind):
        sock = socket.socket()
        self.sockets[sock] = {"kind": kind}
        sock.setblocking(False)
        sock.bind(("127.0.0.1", 0))
        sock.listen(8)
        self.sel.register(sock, selectors.EVENT_READ, self.sockets[sock])
        return sock.getsockname()[1]

    def network(self, sock, state, mask):
        kind = state["kind"]
        if kind in ("http_listener", "pcm_listener"):
            conn, _ = sock.accept()
            new = {"kind": "http" if kind == "http_listener" else "pcm",
                   "input": bytearray(), "accepted_ms": self.ms()}
            self.sockets[conn] = new
            conn.setblocking(False)
            self.sel.register(conn, selectors.EVENT_READ, new)
            require(len(self.sockets) <= 16, "too many fixture connections")
            if new["kind"] == "pcm":
                self.pcm_connections += 1
                require(self.pcm_connections == 1, "Player reopened TCP output")
                self.stamp("pcm_connected")
            return
        if kind == "pcm":
            data = sock.recv(32768)
            if data:
                self.consume(data)
            else:
                self.stamp("pcm_eof")
                self.close_socket(sock)
            return
        if kind == "holding":
            try:
                data = sock.recv(1024)
            except (ConnectionResetError, ConnectionAbortedError):
                data = b""
            require(not data, "unexpected data after stalled HTTP request")
            # The client may close first; keep OUR fd alive without replying.
            self.sel.unregister(sock)
            state["kind"] = "holding_peer_closed"
            self.stamp("stalled_peer_closed")
            return
        if mask & selectors.EVENT_WRITE:
            try:
                count = sock.send(state["output"][:65536])
            except (BrokenPipeError, ConnectionResetError):
                self.stamp("http_peer_closed", source=state["source"])
                self.close_socket(sock)
                return
            require(count > 0, "HTTP send made no progress")
            state["output"] = state["output"][count:]
            if not state["output"]:
                self.stamp("http_body_sent", source=state["source"])
                self.close_socket(sock)
            return
        data = sock.recv(8192)
        require(data, "HTTP connection closed before request")
        state["input"].extend(data)
        require(len(state["input"]) <= 16384, "oversized fixture request")
        if b"\r\n\r\n" not in state["input"]:
            return
        first = bytes(state["input"]).split(b"\r\n", 1)[0].split()
        require(len(first) == 3 and first[0] == b"GET", "fixture expects HTTP GET")
        require(first[1] in (b"/a.mp3", b"/b.mp3"), "unexpected fixture path")
        source = "A" if first[1] == b"/a.mp3" else "B"
        state["source"] = source
        self.result["requests"].append({"source": source, "ms": self.ms()})
        if self.name == "stalled_headers" and source == "A":
            state["kind"] = "holding"
            self.stamp("headers_held", source=source)
            return
        body = self.media[source]
        header = ("HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\n"
                  f"Content-Length: {len(body)}\r\nConnection: close\r\n\r\n").encode()
        state["kind"] = "sending"
        state["output"] = memoryview(header + body)
        self.sel.modify(sock, selectors.EVENT_WRITE, state)

    def consume(self, data):
        at = self.ms()
        require(self.pcm_bytes + len(data) <= 16 * 1024 * 1024, "PCM capture budget exceeded")
        self.pcm_file.write(data)
        self.pcm_hash.update(data)
        self.result["pcm_chunks"].append({"ms": at, "offset": self.pcm_bytes, "bytes": len(data)})
        self.pcm_bytes += len(data)
        self.pcm_pending.extend(data)
        self.segments.append([len(data), at])
        while len(self.pcm_pending) >= WINDOW_BYTES:
            offset = self.pcm_bytes - len(self.pcm_pending)
            block = bytes(self.pcm_pending[:WINDOW_BYTES])
            del self.pcm_pending[:WINDOW_BYTES]
            first, last, remaining = self.segments[0][1], at, WINDOW_BYTES
            while remaining:
                count, last = self.segments[0]
                used = min(count, remaining)
                remaining -= used
                if used == count:
                    self.segments.popleft()
                else:
                    self.segments[0][0] -= used
            self.result["frequency_windows"].append({
                "offset": offset, "frames": FRAMES, "first_ms": first, "last_ms": last,
                **spectrum(block),
            })

    def pipe(self, stream, name):
        data = os.read(stream.fileno(), 65536)
        if not data:
            self.sel.unregister(stream)
            self.pipes.pop(stream)
            if self.buffers[name]:
                self.line(name, bytes(self.buffers[name]), incomplete=True)
                self.buffers[name].clear()
            return
        self.child_bytes += len(data)
        require(self.child_bytes <= 8 * 1024 * 1024, "child output budget exceeded")
        self.buffers[name].extend(data)
        require(len(self.buffers[name]) <= 1024 * 1024, "child line budget exceeded")
        while b"\n" in self.buffers[name]:
            line, _, tail = self.buffers[name].partition(b"\n")
            self.buffers[name] = bytearray(tail)
            self.line(name, bytes(line))

    def line(self, name, raw, incomplete=False):
        text = raw.decode("utf-8", errors="replace")
        self.result[name].append(text)
        if name == "stderr":
            return
        try:
            require(not incomplete, "unterminated stdout line")
            event = json.loads(raw.decode("utf-8"))
            require(isinstance(event, dict) and isinstance(event.get("detail"), dict),
                    "invalid event object")
            require(type(event.get("seq")) is int and type(event.get("ms")) is int,
                    "missing event sequence/timestamp")
            require(isinstance(event.get("event"), str), "missing event name")
        except (ValueError, UnicodeError, RuntimeError, RecursionError) as error:
            self.result["parse_errors"].append({"line": text, "reason": str(error)})
            return
        event["observer_ms"] = self.ms()
        self.result["engine_events"].append(event)
        detail = event["detail"]
        if event["event"] == "ready":
            self.ready = detail
        elif event["event"] == "state":
            self.latest = event
        elif event["event"] == "command_returned":
            require(self.pending is not None and detail == {
                "id": self.pending["id"], "command": self.pending["command"]},
                "unexpected command response")
            self.pending["returned_ms"] = event["ms"]
            self.pending = None
        elif event["event"] == "fatal":
            raise RuntimeError("Rust child reported fatal error")

    def send(self, name):
        require(self.pending is None, "overlapping controller commands")
        self.next_id += 1
        record = {"id": self.next_id, "command": name, "sent_ms": self.ms()}
        self.result["commands"].append(record)
        self.pending = record
        payload = (json.dumps({"id": record["id"], "command": name}) + "\n").encode()
        # One small command in flight: an atomic, nonblocking pipe write.
        require(os.write(self.proc.stdin.fileno(), payload) == len(payload), "short command write")
        self.next_poll = self.ms() + 25
        if name == "drop_b":
            self.finishing = True
            self.proc.stdin.close()
        return record

    def pump(self, auto=True):
        require(self.ms() < 35000, "case wall-clock budget exceeded")
        if self.pending and self.ms() - self.pending["sent_ms"] >= 5000:
            self.result["watchdog_killed"] = True
            self.stamp("command_watchdog", id=self.pending["id"])
            self.proc.kill()
            raise RuntimeError("child command exceeded 5s watchdog")
        if auto and self.ready and not self.finishing and not self.pending and self.ms() >= self.next_poll:
            self.send("snapshot")
        self.poll(.025)
        for state in list(self.sockets.values()):
            if state["kind"] == "http":
                require(self.ms() - state["accepted_ms"] < 2000, "fixture header read timeout")

    def poll(self, timeout):
        for key, mask in self.sel.select(timeout):
            try:
                if isinstance(key.data, str):
                    self.pipe(key.fileobj, key.data)
                else:
                    self.network(key.fileobj, key.data, mask)
            except BlockingIOError:
                continue

    def wait(self, predicate, timeout, reason, auto=True):
        deadline = time.monotonic() + timeout
        while not predicate():
            require(time.monotonic() < deadline, reason)
            require(self.proc.poll() is None or self.pipes, "child exited before observation")
            self.pump(auto)

    def command(self, name):
        self.wait(lambda: self.pending is None, 5, "previous command did not return", auto=False)
        record = self.send(name)
        self.wait(lambda: self.pending is None, 5, f"{name} did not return", auto=False)
        return record

    def called(self, command):
        return next(event for event in self.result["engine_events"]
                    if event["event"] == "command_called" and event["detail"]["id"] == command["id"])

    def state(self):
        return self.latest["detail"] if self.latest else {}

    def observe_until(self, at):
        self.wait(lambda: self.ms() >= at, max(0, (at - self.ms()) / 1000) + 1,
                  "observation did not finish")

    def pure_interval(self, start, end, minimum):
        windows = [w for w in self.result["frequency_windows"]
                   if w["first_ms"] >= start and w["last_ms"] <= end]
        require(len(windows) >= minimum, "insufficient fresh B frequency windows")
        require(all(w["tone"] == "B" for w in windows), "A contamination or B interruption")
        require(windows[-1]["last_ms"] - windows[0]["first_ms"] >= (minimum - 1) * 100,
                "B evidence arrived only as a buffered burst")
        require(all(right["last_ms"] - left["last_ms"] <= 250
                    for left, right in zip(windows, windows[1:])), "B delivery gap exceeded 250ms")
        return len(windows)

    def active_b(self, generation):
        state = self.state()
        require(state["B"]["generation"] == generation and not state["B"]["cancel_requested"]
                and not state["B"]["decoder_joined"] and state["B"]["phase"] == "Decoding",
                "B is not actively decoding")
        require(state["output"]["generation"] == generation and not state["output"]["output_failed"]
                and state["output"]["nonblocking"] is True, "B output generation/failure")
        return state["output"]

    def scenario(self):
        self.wait(lambda: self.ready is not None, 5, "Player did not become ready", auto=False)
        require(self.ready == {"protocol": 1, "sample_rate": RATE, "channels": 2,
                               "format": "s16le", "boundary": "TCP byte-stream delivery"},
                "unexpected PCM/command protocol")
        self.command("play_a")
        if self.name == "playing_audio":
            self.wait(lambda: len(self.result["frequency_windows"]) >= 3 and
                      all(w["tone"] == "A" for w in self.result["frequency_windows"][-3:]),
                      5, "A never delivered confirmed 440Hz audio")
        else:
            self.wait(lambda: any(s["kind"].startswith("holding") for s in self.sockets.values()),
                      5, "A never reached held headers")
            require(not self.state()["A"]["reader_released"] and not self.state()["A"]["worker_exited"],
                    "A released before replacement")
        generation_a = self.state()["A"]["generation"]
        replacement = self.command("replace_b")
        generation_b = self.state()["B"]["generation"]
        require(generation_b > generation_a, "replacement did not create a new generation")
        self.wait(lambda: released(self.state().get("A")), 5, "A resources were not reclaimed")
        release_event = self.latest
        release_ms = release_event["ms"] - self.called(replacement)["ms"]
        self.result["a_release_ms"] = release_ms
        require(0 <= release_ms <= 2000, "A replacement cleanup exceeded 2s")
        if self.name == "stalled_headers":
            held = [sock for sock, s in self.sockets.items() if s["kind"].startswith("holding")]
            require(len(held) == 1 and held[0].fileno() >= 0, "stalled source closed before A release")
            self.stamp("held_source_released_after_a", state_seq=release_event["seq"])
            self.result["source_held_until_a_released"] = True
            self.close_socket(held[0])
        self.wait(lambda: any(w["tone"] == "B" and w["first_ms"] >= replacement["sent_ms"]
                              for w in self.result["frequency_windows"]), 5, "B audio did not appear")
        first_b = next(w for w in self.result["frequency_windows"]
                       if w["tone"] == "B" and w["first_ms"] >= replacement["sent_ms"])
        self.result["b_first_delivery_ms"] = first_b["last_ms"] - replacement["sent_ms"]
        require(self.result["b_first_delivery_ms"] <= 2000, "B transition exceeded 2s")
        stable_start, stable_end = replacement["sent_ms"] + 2000, replacement["sent_ms"] + 3200
        self.observe_until(stable_end)
        self.result["b_stable_windows"] = self.pure_interval(stable_start, stable_end, 8)
        before = self.active_b(generation_b).copy()
        stale = self.command("cancel_a")
        after = self.active_b(generation_b)
        require(after["epoch"] == before["epoch"], "stale A cancellation changed output epoch")
        end = stale["sent_ms"] + 1600
        self.observe_until(end)
        self.result["b_after_stale_windows"] = self.pure_interval(stale["sent_ms"] + 100, end, 10)
        continuing = self.active_b(generation_b)
        require(continuing["nonzero_bytes_written"] > after["nonzero_bytes_written"] + 1000,
                "B writer did not deliver new nonzero bytes after stale cancellation")
        require(continuing["epoch"] == before["epoch"], "B epoch changed after stale cancellation")
        dropping = self.command("drop_b")
        self.wait(lambda: self.proc.poll() is not None and not self.pipes, 5,
                  "child did not exit and close event pipes", auto=False)
        final = self.state()
        require(released(final["A"]) and released(final["B"]), "final sessions not reclaimed")
        drop_call = self.called(dropping)
        joined = next(e for e in self.result["engine_events"] if e["event"] == "drop_joined")
        self.result["b_release_ms"] = self.latest["ms"] - drop_call["ms"]
        self.result["drop_join_ms"] = joined["ms"] - drop_call["ms"]
        require(0 <= self.result["b_release_ms"] <= 2000 and
                0 <= self.result["drop_join_ms"] <= 2000, "B cancellation/Player Drop exceeded 2s")
        # Validate every observed output generation throughout B's active interval.
        begin = next(e for e in self.result["engine_events"]
                     if e["event"] == "state" and e["detail"]["id"] == replacement["id"])["seq"]
        for event in self.result["engine_events"]:
            if event["event"] == "state" and begin <= event["seq"] < drop_call["seq"]:
                require(event["detail"]["output"]["generation"] == generation_b and
                        event["detail"]["B"]["generation"] == generation_b and
                        not event["detail"]["B"]["cancel_requested"], "B generation changed or was cancelled")
        tail = [w for w in self.result["frequency_windows"]
                if w["first_ms"] >= stable_start and w["last_ms"] < dropping["sent_ms"]]
        require(tail and all(w["tone"] == "B" for w in tail), "A reappeared after transition")
        self.result["generations"] = {"A": generation_a, "B": generation_b}
        # Independent session clocks survive replacement without being reset or
        # advanced by another source. These frames measure post-DSP delivery.
        for label in ("A", "B"):
            clocks = [e["detail"][label] for e in self.result["engine_events"]
                      if e["event"] == "state" and e["detail"].get(label)]
            require(all(c["output_boundary"] == "ByteStream"
                        and c["output_generation"] == c["generation"]
                        and c["output_sample_rate"] == RATE
                        and c["output_duration_ns"] == c["output_frames"] * 1_000_000_000 // RATE
                        for c in clocks), "incorrect output clock metadata")
            counts = [c["output_frames"] for c in clocks]
            require(counts == sorted(counts), "session output clock regressed")
        a_after = [e["detail"]["A"]["output_frames"] for e in self.result["engine_events"]
                   if e["event"] == "state" and e["seq"] >= begin]
        require(a_after and len(set(a_after)) == 1, "A clock advanced after replacement")
        require((a_after[0] > 0) if self.name == "playing_audio" else (a_after[0] == 0),
                "A clock counted waiting silence or missed actual media")
        b_at_stale = next(e["detail"]["B"]["output_frames"] for e in self.result["engine_events"]
                          if e["event"] == "state" and e["detail"]["id"] == stale["id"])
        require(final["B"]["output_frames"] > b_at_stale > 0, "B clock did not advance after stale cancel")
        self.result["output_frames"] = {"A": a_after[0], "B": final["B"]["output_frames"],
                                         "B_at_stale_cancel": b_at_stale}

    def run(self):
        try:
            pcm_path = self.dest / f"replacement-{self.name}.s16le"
            self.result["pcm_file"] = pcm_path.name
            self.pcm_file = pcm_path.open("wb")
            http_port, pcm_port = self.listen("http_listener"), self.listen("pcm_listener")
            self.proc = subprocess.Popen([str(self.exe), f"http://127.0.0.1:{http_port}/a.mp3",
                                          f"http://127.0.0.1:{http_port}/b.mp3",
                                          f"tcp-connect:127.0.0.1:{pcm_port}"],
                                         stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            for name in ("stdin", "stdout", "stderr"):
                stream = getattr(self.proc, name)
                os.set_blocking(stream.fileno(), False)
                if name != "stdin":
                    self.pipes[stream] = name
                    self.buffers[name] = bytearray()
                    self.sel.register(stream, selectors.EVENT_READ, name)
            self.scenario()
        except Exception as error:
            self.result["errors"].append(f"{type(error).__name__}: {error}")
        finally:
            def cleanup(label, action):
                try:
                    action()
                except Exception as error:
                    self.result["errors"].append(f"{label}: {type(error).__name__}: {error}")

            # Child exit precedes fixture closure, including on failure.
            if self.proc is not None:
                if self.proc.poll() is None:
                    self.result["cleanup_killed"] = True
                    self.stamp("cleanup_kill")
                    if self.pending and self.ms() - self.pending["sent_ms"] >= 5000:
                        self.result["watchdog_killed"] = True
                        self.stamp("command_watchdog", id=self.pending["id"])
                    cleanup("kill child", self.proc.kill)
                cleanup("reap child within 5s", lambda: self.proc.wait(timeout=5))
                drain_until = time.monotonic() + 2
                while time.monotonic() < drain_until and (self.pipes or
                        any(s["kind"] == "pcm" for s in self.sockets.values())):
                    try:
                        self.poll(.025)
                    except Exception as error:
                        self.result["errors"].append(f"drain: {type(error).__name__}: {error}")
                        break
                if self.pipes:
                    self.result["errors"].append("event pipes did not reach EOF")
                for stream in (self.proc.stdin, self.proc.stdout, self.proc.stderr):
                    cleanup("close child pipe", stream.close)
                self.result["exit_code"] = self.proc.returncode
            for sock in list(self.sockets):
                cleanup("close fixture socket", lambda sock=sock: self.close_socket(sock))
            cleanup("close selector", self.sel.close)
            if self.pcm_file is not None:
                cleanup("close PCM artifact", self.pcm_file.close)
            self.result["pcm_bytes"] = self.pcm_bytes
            self.result["pcm_sha256"] = self.pcm_hash.hexdigest()
        try:
            require(not self.result["errors"], "scenario failed")
            require(self.result.get("exit_code") == 0, "child exit was not zero")
            require(not self.result["parse_errors"] and not self.result["stderr"], "malformed stdout or stderr")
            require(not self.result["watchdog_killed"] and not self.result["cleanup_killed"], "watchdog cleanup is failure")
            require(self.pcm_connections == 1 and self.pcm_bytes > 0 and self.pcm_bytes % 4 == 0,
                    "invalid stereo PCM stream")
            require(collections.Counter(r["source"] for r in self.result["requests"]) == {"A": 1, "B": 1},
                    "expected exactly one HTTP request per source")
            events = self.result["engine_events"]
            require([e["seq"] for e in events] == list(range(1, len(events) + 1)), "event sequence gap")
            require(all(a["ms"] <= b["ms"] for a, b in zip(events, events[1:])), "event clock regressed")
            for command in self.result["commands"]:
                matching = [e for e in events if e["detail"].get("id") == command["id"]]
                names = [e["event"] for e in matching]
                expected = (["command_called", "drop_started", "drop_joined", "state", "command_returned"]
                            if command["command"] == "drop_b" else
                            ["command_called", "state", "command_returned"])
                require(names == expected, "incomplete command/response sequence")
                require(matching[0]["detail"]["command"] == command["command"] and
                        matching[-1]["detail"]["command"] == command["command"],
                        "command name did not match response")
            require(all(not e["detail"]["output"]["output_failed"] for e in events
                        if e["event"] == "state" and e["detail"].get("output")), "output reported failure")
            self.result["pass"] = True
        except Exception as error:
            self.result["errors"].append(f"validation: {error}")
        return self.result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=pathlib.Path, default=ROOT / "target/debug/replacement")
    parser.add_argument("--output", type=pathlib.Path, default=ROOT / "results")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    report = {"profile": "public-player-replacement-v1", "pass": False, "cases": [],
              "errors": [], "fixtures": {}, "sample_rate": RATE, "channels": 2,
              "pcm_format": "s16le", "transition_limit_ms": 2000, "join_limit_ms": 2000,
              "command_watchdog_ms": 5000, "window_frames": FRAMES,
              "frequency_gate": {"min_rms": 100, "min_target_fraction": .8, "max_other_fraction": .01},
              "boundary": "Loopback TCP delivery only; no audible-consumption or other-output claim."}
    try:
        require(sys.platform.startswith("linux"), "this CI probe requires Linux pipe selectors")
        exe = args.exe.resolve()
        report["executable_sha256"] = sha256(exe)
        media = {}
        for label, frequency in (("A", 440), ("B", 880)):
            path = args.output / f"replacement-{label}-{frequency}.mp3"
            media[label] = fixture(path, frequency)
            report["fixtures"][label] = {"file": path.name, "frequency": frequency, "seconds": 30,
                                          "sha256": hashlib.sha256(media[label]).hexdigest()}
        for name in ("playing_audio", "stalled_headers"):
            case = Case(name, exe, media, args.output)
            # Attach evidence before execution, including unexpected cleanup failures.
            report["cases"].append(case.result)
            case.run()
        report["pass"] = all(case["pass"] for case in report["cases"])
    except Exception as error:
        report["errors"].append(f"{type(error).__name__}: {error}")
    finally:
        path = args.output / "replacement.json"
        path.write_text(json.dumps(report, indent=2, allow_nan=False), encoding="utf-8")
    print(json.dumps({"pass": report["pass"], "artifact": str(path),
                      "cases": [{"case": c["case"], "pass": c["pass"]} for c in report["cases"]]}))
    return 0 if report["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
