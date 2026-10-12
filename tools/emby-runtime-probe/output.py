#!/usr/bin/env python3
"""Linux CI only: real FIFO/TCP/Unix backpressure through public Player Drop.

Integrate beside run.py. No output receiver ever reads. Expected total runtime
is under two minutes, including the shared 120-second fixture's 30s build limit.
"""
import hashlib
import http.server
import json
import os
import platform
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BLOCK_TIMEOUT = {"fifo": 10.0, "tcp": 30.0, "unix": 10.0}
WATCHDOG_TIMEOUT = 5.0
JOIN_TIMEOUT_MS = 2000
STABLE_MS = 300
RECEIVE_BUFFER = 4096
RELEASE_KEYS = ("cancel_requested", "reader_released",
                "decoder_joined", "worker_exited")
# Only used if SIGKILL itself cannot be reaped: never close that child's receiver.
UNREAPED_RESOURCES = []


def blocked(event):
    detail = event.get("detail", {})
    return (event.get("event") == "output"
            and detail.get("nonblocking") is True
            and all(type(detail.get(key)) is int and detail[key] > 0
                    for key in ("backpressure_events", "bytes_written",
                                "nonzero_bytes_written")))


def stable_window(events):
    """Pair each session clock with its preceding output snapshot.

    Require an unchanged frame/byte count and strictly growing WouldBlock count
    throughout 300ms of child timestamps; an initial transient stall is not enough.
    """
    output, window = None, []
    for event in events:
        if event["event"] == "output":
            output = event
        if event["event"] != "session":
            continue
        frames = event["detail"].get("output_frames")
        if (output is None or not blocked(output) or type(frames) is not int
                or frames <= 0 or output["detail"].get("output_failed") is not False
                or output["detail"].get("writer_exited") is not False):
            window = []
            continue
        detail = output["detail"]
        sample = {"ms": event["ms"], "output_ms": output["ms"],
                  "frames": frames, "bytes_written": detail["bytes_written"],
                  "nonzero_bytes_written": detail["nonzero_bytes_written"],
                  "backpressure_events": detail["backpressure_events"]}
        if window and (any(sample[key] != window[-1][key] for key in
                           ("frames", "bytes_written", "nonzero_bytes_written"))
                       or sample["backpressure_events"] <= window[-1]["backpressure_events"]):
            window = []
        window.append(sample)
    return (window if len(window) >= 3
            and window[-1]["ms"] - window[0]["ms"] >= STABLE_MS
            and window[-1]["output_ms"] - window[0]["output_ms"] >= STABLE_MS else [])


class Receiver:
    def __init__(self, kind, base, stamp):
        self.kind, self.base, self.stamp = kind, base, stamp
        self.listener = self.connection = None
        self.fd = None
        self.metadata = {"kind": kind, "read_calls": 0,
                         "kept_open_until_process_exit": False}

    def prepare(self):
        if self.kind == "fifo":
            path = self.base / "pcm.fifo"
            os.mkfifo(path)
            self.fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK)
            self.metadata["mode"] = "O_RDONLY|O_NONBLOCK"
            config = "fifo:" + str(path)
            self.opened()
        else:
            family = socket.AF_INET if self.kind == "tcp" else socket.AF_UNIX
            self.listener = socket.socket(family, socket.SOCK_STREAM)
            self.listener.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, RECEIVE_BUFFER)
            address = (("127.0.0.1", 0) if self.kind == "tcp"
                       else str(self.base / "pcm.sock"))
            self.listener.bind(address)
            self.listener.listen(1)
            self.listener.settimeout(3.0)
            address = self.listener.getsockname()
            target = f"{address[0]}:{address[1]}" if self.kind == "tcp" else address
            # Player connects to our listener, so we control both receive buffers.
            config = self.kind + "-connect:" + target
            self.metadata.update({"role": "harness-listener/player-connect",
                                  "requested_rcvbuf": RECEIVE_BUFFER,
                                  "listener_rcvbuf": self.listener.getsockopt(
                                      socket.SOL_SOCKET, socket.SO_RCVBUF)})
            self.stamp("receiver_listening", fd=self.listener.fileno(), config=config)
        self.metadata["output_config"] = config
        return config

    def accept(self):
        if self.listener is not None:
            self.connection, _ = self.listener.accept()
            self.connection.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, RECEIVE_BUFFER)
            self.connection.setblocking(False)
            self.fd = self.connection.fileno()
            self.metadata["accepted_rcvbuf"] = self.connection.getsockopt(
                socket.SOL_SOCKET, socket.SO_RCVBUF)
            self.opened()

    def opened(self):
        stat = os.fstat(self.fd)
        self.metadata.update({"fd": self.fd, "device": stat.st_dev, "inode": stat.st_ino})
        self.metadata["opened_ms"] = self.stamp("receiver_opened", fd=self.fd)["ms"]

    def verify_after_exit(self, exit_ms):
        if self.fd is not None:
            stat = os.fstat(self.fd)
            alive = (stat.st_dev == self.metadata["device"]
                     and stat.st_ino == self.metadata["inode"])
            observed = self.stamp("receiver_alive_after_process_exit",
                                  fd=self.fd, same_descriptor=alive)
            self.metadata.update({"child_exit_observed_ms": exit_ms,
                                  "alive_checked_ms": observed["ms"],
                                  "kept_open_until_process_exit": alive})

    def close(self):
        try:
            if self.connection is not None:
                self.connection.close()
            elif self.fd is not None:
                os.close(self.fd)
            if self.fd is not None:
                self.metadata["closed_ms"] = self.stamp("receiver_closed")["ms"]
        finally:
            if self.listener is not None:
                self.listener.close()


def run_case(kind, exe_path, media, base, common, setup_error, evidence):
    start = time.monotonic()
    raw_stdout, stderr, events, parse_errors = [], [], [], []
    observer_events, http_requests, server_errors, errors = [], [], [], []
    proc = server = server_thread = None
    collectors, parsed_lines = [], 0
    server_stop = threading.Event()
    watchdog_killed = False
    drop_sent = exit_observed = None
    window = []

    def stamp(event, **detail):
        value = {"ms": round((time.monotonic() - start) * 1000, 3),
                 "event": event, **detail}
        observer_events.append(value)
        return value

    receiver = Receiver(kind, base, stamp)
    # Share the evidence as it arrives, including an unexpected cleanup failure.
    evidence.update({"engine_events": events, "raw_stdout": raw_stdout,
                     "observer_events": observer_events, "stderr": stderr,
                     "errors": errors, "parse_errors": parse_errors,
                     "server_errors": server_errors, "http_requests": http_requests,
                     "output_receiver": receiver.metadata})

    class Handler(http.server.BaseHTTPRequestHandler):
        def setup(self):
            self.request.settimeout(2.0)
            super().setup()

        def log_message(self, *args):
            pass

        def do_GET(self):
            http_requests.append({"path": self.path, "content_length": len(media)})
            try:
                self.send_response(200)
                self.send_header("Content-Type", "audio/mpeg")
                self.send_header("Content-Length", str(len(media)))
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(media)
                self.wfile.flush()
                stamp("http_body_sent", bytes=len(media))
            except OSError as error:
                server_errors.append(repr(error))

    class Server(http.server.HTTPServer):
        def handle_error(self, request, client_address):
            server_errors.append(repr(sys.exc_info()[1]))

    def serve():
        try:
            while not server_stop.is_set():
                server.handle_request()
        except Exception as error:
            server_errors.append(repr(error))

    def collect(source, sink):
        try:
            for line in source:
                sink.append(line.rstrip("\n"))
        except Exception as error:
            errors.append({"collector": repr(error)})

    def drain():
        nonlocal parsed_lines
        for line in raw_stdout[parsed_lines:]:
            parsed_lines += 1
            try:
                event = json.loads(line)
                if (not isinstance(event, dict) or type(event.get("ms")) is not int
                        or event["ms"] < 0 or not isinstance(event.get("event"), str)
                        or not isinstance(event.get("detail"), dict)):
                    raise ValueError("invalid child event schema")
                events.append(event)
            except (ValueError, TypeError, RecursionError) as error:
                parse_errors.append({"line": parsed_lines, "raw": line, "error": str(error)})

    try:
        if setup_error:
            raise RuntimeError(setup_error)
        deadline = start + BLOCK_TIMEOUT[kind]
        config = receiver.prepare()
        server = Server(("127.0.0.1", 0), Handler)
        server.timeout = 0.1
        server_thread = threading.Thread(target=serve, daemon=True)
        server_thread.start()
        url = f"http://127.0.0.1:{server.server_port}/finite_no_range.mp3"
        proc = subprocess.Popen([str(exe_path), url, config], stdin=subprocess.PIPE,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                text=True, encoding="utf-8")
        stamp("process_started", pid=proc.pid)
        for source, sink in ((proc.stdout, raw_stdout), (proc.stderr, stderr)):
            thread = threading.Thread(target=collect, args=(source, sink), daemon=True)
            thread.start()
            collectors.append(thread)
        receiver.accept()
        while time.monotonic() < deadline and proc.poll() is None:
            drain()
            if parse_errors or errors or stderr or server_errors:
                raise RuntimeError("error while observing output; see raw evidence")
            window = stable_window(events)
            if window:
                break
            time.sleep(0.025)
        if not window:
            raise RuntimeError("no 300ms frozen media clock with growing real backpressure")
        drop_sent = stamp("drop_sent", stable_window_ms=window[-1]["ms"] - window[0]["ms"])["ms"]
        proc.stdin.write("drop\n")
        proc.stdin.flush()
        # Passing is decided from <=2s child release/Drop timestamps below.
        # This 5s wait is only the point at which a hung child is killed.
        proc.wait(timeout=WATCHDOG_TIMEOUT)
    except Exception as error:
        errors.append({"run": repr(error)})
    finally:
        if proc is not None:
            if proc.poll() is None:
                watchdog_killed = True
                stamp("watchdog_kill", reason="drop_timeout" if drop_sent is not None
                      else "observation_or_setup_failure")
                try:
                    proc.kill()
                    proc.wait(timeout=1.0)
                except (OSError, subprocess.TimeoutExpired) as error:
                    errors.append({"kill_reap": repr(error)})
            if proc.poll() is not None:
                exit_observed = stamp("process_exited", exit_code=proc.returncode)["ms"]
                try:
                    receiver.verify_after_exit(exit_observed)
                except OSError as error:
                    errors.append({"receiver_lifetime": repr(error)})
        # Collect to EOF before the final parse: drop_joined/final session are last.
        join_deadline = time.monotonic() + 1.0
        for thread in collectors:
            thread.join(timeout=max(0.0, join_deadline - time.monotonic()))
        if proc is not None and all(not t.is_alive() for t in collectors):
            for stream in (proc.stdin, proc.stdout, proc.stderr):
                try:
                    stream.close()
                except OSError as error:
                    errors.append({"pipe_close": repr(error)})
        server_stop.set()
        if server_thread is not None and server_thread.ident is not None:
            server_thread.join(timeout=2.5)
        if server is not None:
            server.server_close()
        if proc is None or proc.poll() is not None:
            try:
                receiver.close()
            except OSError as error:
                errors.append({"receiver_close": repr(error)})
        else:
            UNREAPED_RESOURCES.append((proc, receiver))
            errors.append({"cleanup": "child unreaped; receiver deliberately retained"})
        drain()

    outputs = [e for e in events if e["event"] == "output"]
    sessions = [e for e in events if e["event"] == "session"]
    called = [e for e in events if e["event"] == "stop_called"
              and e["detail"].get("command") == "drop"]
    joined = [e for e in events if e["event"] == "drop_joined"]
    started = [e for e in events if e["event"] == "drop_started"]
    drop_called, drop_joined = next(iter(called), None), next(iter(joined), None)
    final_session = sessions[-1] if sessions else None
    released = [e for e in sessions if drop_called and e["ms"] >= drop_called["ms"]
                and all(e["detail"].get(key) is True for key in RELEASE_KEYS)]
    drop_latency = drop_joined["ms"] - drop_called["ms"] if drop_joined and drop_called else None
    release_latency = released[0]["ms"] - drop_called["ms"] if released else None
    exit_latency = exit_observed - drop_sent if exit_observed is not None and drop_sent is not None else None
    first_blocked = next((e for e in outputs if blocked(e)), None)
    last_detail = outputs[-1]["detail"] if outputs else {}
    frames = [e["detail"].get("output_frames") for e in sessions]
    valid_frames = bool(frames) and all(type(f) is int and f >= 0 for f in frames)
    clock_schema = valid_frames and all(
        e["detail"].get("output_boundary") == "ByteStream"
        and e["detail"].get("output_generation") == e["detail"].get("generation")
        and e["detail"].get("output_sample_rate") == 44100
        and e["detail"].get("output_duration_ns") == e["detail"]["output_frames"] * 1_000_000_000 // 44100
        for e in sessions)
    frozen = [e for e in sessions if window and e["ms"] >= window[0]["ms"]]
    fifo_clocks = [e for e in sessions if first_blocked and drop_called
                   and first_blocked["ms"] + 50 <= e["ms"] < drop_called["ms"]]
    byte_count = last_detail.get("bytes_written", 0)
    checks = {
        "backpressure_blocked": first_blocked is not None,
        "blocked_nonblocking": first_blocked is not None and first_blocked["detail"].get("nonblocking") is True,
        "nonzero_bytes_written": bool(outputs) and blocked(outputs[-1]),
        "stable_300ms_with_growing_backpressure_before_drop": bool(window) and drop_called is not None
            and window[-1]["ms"] < drop_called["ms"],
        "session_present": bool(sessions),
        "clock_schema": clock_schema,
        "clock_monotonic": valid_frames and frames == sorted(frames),
        "clock_excludes_silence": valid_frames and type(byte_count) is int and 0 < frames[-1] * 4 <= byte_count,
        "clock_frozen_while_blocked_and_after_drop": len(frozen) >= 3
            and all(e["detail"].get("output_frames") == window[0]["frames"] for e in frozen),
        "fifo_initial_clock_frozen": kind != "fifo" or (len(fifo_clocks) >= 3
            and all(e["detail"].get("output_frames") == frames[-1] for e in fifo_clocks)),
        "final_session_released": final_session is not None
            and all(final_session["detail"].get(key) is True for key in RELEASE_KEYS),
        "release_latency": release_latency is not None and 0 <= release_latency <= JOIN_TIMEOUT_MS,
        "drop_latency": drop_latency is not None and 0 <= drop_latency <= JOIN_TIMEOUT_MS,
        "exit_latency": exit_latency is not None and 0 <= exit_latency <= JOIN_TIMEOUT_MS,
        "drop_event_sequence": len(called) == len(started) == len(joined) == 1
            and drop_called["ms"] <= started[0]["ms"] <= drop_joined["ms"]
            and final_session is not None and final_session["ms"] >= drop_joined["ms"],
        "exit_zero": proc is not None and proc.returncode == 0,
        "no_watchdog": not watchdog_killed,
        "no_errors": not errors,
        "no_stderr": not stderr,
        "no_server_errors": not server_errors,
        "no_parse_errors": not parse_errors,
        "one_http_request": len(http_requests) == 1,
        "collectors_joined": len(collectors) == 2 and all(not t.is_alive() for t in collectors),
        "server_joined": server_thread is not None and not server_thread.is_alive(),
        "receiver_kept_open_until_process_exit": receiver.metadata["kept_open_until_process_exit"],
        "receiver_closed_after_exit": exit_observed is not None
            and receiver.metadata.get("closed_ms", -1) >= exit_observed,
        "receiver_never_read": receiver.metadata["read_calls"] == 0,
        "small_socket_receive_buffers": kind == "fifo" or all(
            0 < receiver.metadata.get(key, 0) <= RECEIVE_BUFFER * 2
            for key in ("listener_rcvbuf", "accepted_rcvbuf")),
        "no_output_failure": bool(outputs) and all(e["detail"].get("output_failed") is False
            and e["detail"].get("writer_exited") is False for e in outputs),
    }
    summary = {**common, "case": kind, "result": "Pass" if all(checks.values()) else "Fail",
               "checks": checks, "blocked_event": first_blocked,
               "last_output_event": outputs[-1] if outputs else None,
               "drop_called": drop_called, "drop_joined": drop_joined,
               "drop_join_latency_ms": drop_latency, "release_latency_ms": release_latency,
               "exit_latency_ms": exit_latency, "drop_sent_observer_ms": drop_sent,
               "final_session": final_session, "session_count": len(sessions),
               "output_frames": frames[-1] if frames else None,
               "blocked_clock_samples": len(frozen), "watchdog_killed": watchdog_killed,
               "exit_code": proc.returncode if proc else None, "http_request_count": len(http_requests),
               "elapsed_ms": round((time.monotonic() - start) * 1000, 3)}
    evidence.update({"summary": summary, "stable_window": window,
                "note": "Receiver never reads and remains open through child exit, including watchdog cleanup. "
                        "Counters/media clock describe byte-stream delivery, not audible consumption. "
                        "A watchdog kill always fails; Drop and release each require <=2000ms."})
    if kind == "fifo":
        evidence["fifo_reader_fd"] = receiver.metadata
    return evidence


def run_harness(exe_path):
    results_dir = ROOT / "results"
    results_dir.mkdir(parents=True, exist_ok=True)
    common = {"fixture_sha256": None, "executable_sha256": None,
              "executable": str(exe_path),
              "fixture_seconds": 120, "platform": platform.platform(),
              "pcm": "44100 Hz, stereo s16le; delivery only",
              "expected_total_runtime": "<120 seconds"}
    failed = False
    with tempfile.TemporaryDirectory(prefix="emby-output-") as directory:
        base = Path(directory)
        media, setup_error = b"", None
        try:
            if platform.system() != "Linux":
                raise RuntimeError("Linux-only output harness")
            common["executable_sha256"] = hashlib.sha256(exe_path.read_bytes()).hexdigest()
            sys.path.insert(0, str(ROOT))
            from run import mp3
            media = mp3(base / "tone-120s.mp3", 120)
            common["fixture_sha256"] = hashlib.sha256(media).hexdigest()
            common["fixture_bytes"] = len(media)
            (results_dir / "tone-output-120s.mp3").write_bytes(media)
        except Exception as error:
            setup_error = repr(error)
        for kind in BLOCK_TIMEOUT:
            evidence = {"summary": {**common, "case": kind, "result": "Fail"}, "errors": []}
            try:
                case_base = base / kind
                case_base.mkdir()
                run_case(kind, exe_path, media, case_base, common, setup_error, evidence)
            except Exception as error:
                evidence["summary"]["result"] = "Fail"
                evidence["errors"].append({"unexpected_case_error": repr(error)})
            (results_dir / f"output-{kind}.json").write_text(
                json.dumps(evidence, indent=2), encoding="utf-8")
            print(json.dumps(evidence["summary"]), flush=True)
            failed |= evidence["summary"]["result"] != "Pass"
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(run_harness(ROOT / "target/debug/emby-runtime-probe"))
