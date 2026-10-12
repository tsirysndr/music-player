"""Bounded loopback experiments against the unmodified public Rockbox Player.
Exit success means evidence was collected, NOT that product gates passed.
"""
import argparse
import hashlib
import http.server
import json
import pathlib
import platform
import socket
import subprocess
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parent


def mp3(path, seconds):
    subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
                    "-f", "lavfi", "-i", "sine=frequency=440:sample_rate=44100",
                    "-t", str(seconds), "-ac", "2", "-codec:a", "libmp3lame",
                    "-b:a", "128k", "-write_xing", "0", "-id3v2_version", "0", str(path)],
                   check=True, timeout=30)
    return path.read_bytes()


def run_case(exe, name, media, short_media, dest):
    start = time.monotonic()
    events, requests, pcm = [], [], []
    requested, stalled, release = threading.Event(), threading.Event(), threading.Event()
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(20)
    port = listener.getsockname()[1]

    def stamp(event, **data):
        events.append({"ms": round((time.monotonic() - start) * 1000, 2), "event": event, **data})

    class Handler(http.server.BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *args):
            pass

        def do_GET(self):
            requests.append({"range": self.headers.get("Range"),
                             "test_header_present": self.headers.get("X-Fixture-Auth") == "synthetic-sentinel"})
            stamp("request_received")
            requested.set()
            try:
                if name == "late_headers":
                    stalled.set()
                    release.wait(15)
                if name == "header_required":
                    self.send_response(401)
                    self.send_header("Content-Length", "0")
                    self.end_headers()
                    return
                data = short_media if name == "short_eof" else media
                if name == "fake_audio":
                    data = b"<html>upstream error despite audio MIME</html>"
                chunked = name in ("chunked", "body_stall", "short_eof")
                self.send_response(200)  # Range deliberately ignored
                self.send_header("Content-Type", "audio/mpeg")
                self.send_header("Connection", "close")
                self.send_header("Transfer-Encoding" if chunked else "Content-Length",
                                 "chunked" if chunked else str(len(data)))
                self.end_headers()
                if name == "body_stall":
                    data = data[:96000]
                if chunked:
                    for offset in range(0, len(data), 4096):
                        block = data[offset:offset + 4096]
                        self.wfile.write(("%X\r\n" % len(block)).encode() + block + b"\r\n")
                        self.wfile.flush()
                    if name == "body_stall":
                        stamp("body_stalled", bytes_sent=len(data))
                        stalled.set()
                        release.wait(15)
                    self.wfile.write(b"0\r\n\r\n")
                else:
                    self.wfile.write(data)
                self.wfile.flush()
                stamp("response_finished")
            except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError, OSError) as error:
                stamp("server_io_error", error=type(error).__name__)
            finally:
                self.close_connection = True
                stamp("handler_exited")

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    server.daemon_threads = True
    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    server_thread.start()

    def consume():
        try:
            conn, _ = listener.accept()
            with conn:
                conn.settimeout(15)
                while True:
                    data = conn.recv(8192)
                    if not data:
                        break
                    pcm.append({"ms": round((time.monotonic() - start) * 1000, 2),
                                "bytes": len(data), "nonzero_bytes": sum(b != 0 for b in data)})
            stamp("pcm_closed")
        except (OSError, TimeoutError) as error:
            stamp("pcm_error", error=type(error).__name__)

    consumer = threading.Thread(target=consume, daemon=True)
    consumer.start()
    proc = subprocess.Popen([str(exe), f"http://127.0.0.1:{server.server_port}/{name}.mp3",
                             f"tcp-connect:127.0.0.1:{port}"],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    engine_events, errors = [], []

    def collect():
        for line in proc.stdout:
            try:
                value = json.loads(line)
                value["observer_ms"] = round((time.monotonic() - start) * 1000, 2)
                engine_events.append(value)
            except json.JSONDecodeError:
                errors.append(line.rstrip())

    def collect_errors():
        errors.extend(line.rstrip() for line in proc.stderr)

    readers = [threading.Thread(target=collect), threading.Thread(target=collect_errors)]
    for reader in readers:
        reader.start()
    killed = False
    stop_ms = None
    try:
        if not requested.wait(15):
            raise RuntimeError("engine never requested fixture")
        if name in ("late_headers", "body_stall") and not stalled.wait(5):
            raise RuntimeError("fixture did not enter intended stall")
        observation = {"late_headers": .25, "body_stall": 8, "short_eof": 4,
                       "fake_audio": 1, "header_required": 1}.get(name, 3)
        time.sleep(observation)
        stop_ms = (time.monotonic() - start) * 1000
        stamp("stop_sent")
        proc.stdin.write("stop\n")
        proc.stdin.flush()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            killed = True
            stamp("watchdog_kill")
            proc.kill()  # only the owned experiment child, never a product stop
            proc.wait(timeout=5)
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait(timeout=5)
        release.set()
        server.shutdown()
        server.server_close()
        consumer.join(timeout=3)
        listener.close()
        for reader in readers:
            reader.join(timeout=3)
        proc.stdin.close()
        proc.stdout.close()
        proc.stderr.close()
    nonzero = [entry for entry in pcm if entry["nonzero_bytes"]]
    joined = next((e for e in engine_events if e["event"] == "drop_joined"), None)
    stopped_call = next((e for e in engine_events if e["event"] == "stop_called"), None)
    summary = {"case": name, "exit_code": proc.returncode, "watchdog_killed": killed,
               "request_count": len(requests), "nonzero_pcm_bytes": sum(e["nonzero_bytes"] for e in pcm),
               "first_nonzero_pcm_ms": nonzero[0]["ms"] if nonzero else None,
               "last_nonzero_after_stop_ms": round(nonzero[-1]["ms"] - stop_ms, 2) if nonzero else None,
               "drop_joined": joined is not None,
               "drop_join_after_stop_ms": joined["ms"] - stopped_call["ms"] if joined and stopped_call else None,
               "internal_cancel_gate": "Fail" if killed else "Unverified",
               "boundary": "TCP delivery only; no reader-join event or native consumption observation"}
    result = {"summary": summary, "fixture_events": events, "engine_events": engine_events,
              "requests": requests, "pcm": pcm, "stderr": errors}
    (dest / f"{name}.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(summary), flush=True)
    return summary


def main():
    parser = argparse.ArgumentParser()
    suffix = ".exe" if platform.system() == "Windows" else ""
    parser.add_argument("--exe", type=pathlib.Path, default=ROOT / ("target/debug/emby-engine-baseline" + suffix))
    parser.add_argument("--cases", nargs="+", default=["finite_no_range", "chunked", "late_headers",
                                                        "body_stall", "header_required", "fake_audio", "short_eof"])
    parser.add_argument("--output", type=pathlib.Path, default=ROOT / "results")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    media = mp3(args.output / "tone-30s.mp3", 30)
    short_media = mp3(args.output / "tone-2s.mp3", 2)
    summaries = [run_case(args.exe.resolve(), name, media, short_media, args.output) for name in args.cases]
    report = {"profile": "v1", "platform": platform.platform(),
              "executable_sha256": hashlib.sha256(args.exe.read_bytes()).hexdigest(),
              "media_sha256": hashlib.sha256(media).hexdigest(), "cases": summaries,
              "note": "Harness completion is not product gate success; all real Emby/native/FIFO tests remain unverified."}
    (args.output / "summary.json").write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
