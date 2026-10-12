"""Verify raw evidence, not just the harness's capability labels."""
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
report = json.loads((root / "summary.json").read_text())
names = {"finite_no_range", "chunked", "late_headers", "body_stall", "header_required",
         "header_positive", "fake_audio", "short_eof", "late_headers_pause", "late_headers_drop", "body_stall_pause", "redirect", "partial_content", "truncated_body"}
assert len(report["cases"]) == len(names)
cases = {c["case"]: c for c in report["cases"]}
assert cases.keys() == names
assert hashlib.sha256((root / "tone-30s.mp3").read_bytes()).hexdigest() == report["media_sha256"]
print("| Case | Reader + codec join after cancel | Nonzero PCM bytes |")
print("| --- | ---: | ---: |")
for name, summary in cases.items():
    raw = json.loads((root / (name + ".json")).read_text())
    assert raw["summary"] == summary
    events = raw["engine_events"]
    assert not raw["stderr"], (name, raw["stderr"])
    assert summary["exit_code"] == 0 and not summary["watchdog_killed"], name
    assert not any(e["event"] == "watchdog_kill" for e in raw["fixture_events"]), name
    stop = next(e for e in events if e["event"] in ("stop_called", "pause_called"))
    joined = next(e for e in events if e["event"] == "drop_joined")
    sessions = [e for e in events if e["event"] == "session"]
    # Monotonic session evidence: once true, sticky flags never regress to false.
    generations = {e["detail"]["generation"] for e in sessions}
    assert generations == {1}, (name, generations)
    sticky_flags = ("reader_released", "decoder_joined", "worker_exited", "cancel_requested")
    observed = {flag: False for flag in sticky_flags}
    for session in sessions:
        for flag in sticky_flags:
            if session["detail"][flag]:
                observed[flag] = True
            else:
                assert not observed[flag], (name, flag, session["ms"])
    complete = next(e for e in sessions if e["ms"] >= stop["ms"] and
                    all(e["detail"][k] for k in ("reader_released", "decoder_joined", "worker_exited")))
    latency = complete["ms"] - stop["ms"]
    assert 0 <= latency <= 2000, (name, latency)
    assert summary["cancel_join_ms"] == latency and summary["internal_cancel_gate"] == "Pass"
    assert summary["drop_joined"] and summary["drop_join_after_stop_ms"] == joined["ms"] - stop["ms"]
    assert summary["request_count"] == len(raw["requests"]) == (2 if name == "redirect" else 1), name
    assert all(r["range"] is None for r in raw["requests"]), name
    pcm = sum(e["nonzero_bytes"] for e in raw["pcm"])
    assert pcm == summary["nonzero_pcm_bytes"]
    if name in {"finite_no_range", "chunked", "header_positive", "body_stall", "body_stall_pause", "short_eof", "truncated_body", "redirect"}:
        assert pcm > 4096, (name, "missing real audio")
        assert summary["first_nonzero_pcm_ms"] < 2000, name
    else:
        assert pcm == 0, (name, "unexpected audio")
    if name == "redirect":
        assert [r["path"] for r in raw["requests"]] == ["/redirect.mp3", "/finite_no_range.mp3"]
        assert all(r["test_header_present"] for r in raw["requests"]), "same-origin auth lost"
        assert sum(e["event"] == "redirect_sent" for e in raw["fixture_events"]) == 1
        assert not any(e["detail"]["phase"] == "Failed" for e in sessions), "redirect failed"
    if name == "header_positive":
        assert raw["requests"][0]["test_header_present"]
    if name == "header_required":
        assert not raw["requests"][0]["test_header_present"]
        assert any(e["detail"]["transport_terminal"] == "HttpStatus(401)" for e in sessions)
        assert any(e["detail"]["phase"] == "Failed" for e in sessions)
    if name in {"partial_content", "truncated_body"}:
        expected = {"partial_content": "HttpStatus(206)",
                    "truncated_body": "NetworkError"}[name]
        before = [e["detail"] for e in sessions if e["ms"] < stop["ms"]]
        assert any(s["transport_terminal"] == expected for s in before), (name, expected)
        assert any(s["phase"] == "Failed" for s in before), (name, "error hidden")
        assert not any(s["phase"] == "EndUnconfirmed" for s in before), (name, "error treated as EOF")
    if name.startswith("late_headers") or name.startswith("body_stall"):
        # Must have cancelled a still-live source; natural completion isn't cancellation proof.
        before = [e for e in sessions if e["ms"] < stop["ms"]]
        assert before and not before[-1]["detail"]["worker_exited"], name
        assert complete["detail"]["cancel_requested"], name
        assert complete["detail"]["transport_terminal"] == "Cancelled", name
    if name == "short_eof":
        assert any(e["detail"]["phase"] == "EndUnconfirmed" and
                   e["detail"]["decoder_status"] == 0 for e in sessions if e["ms"] < stop["ms"])
        positions = [e["detail"]["position_ms"] for e in events if e["event"] == "status" and
                     3000 <= e["ms"] < stop["ms"]]
        assert positions and min(positions) >= 1500, (name, "checkpoint disappeared")
    print(f"| {name} | {latency} ms | {pcm} |")
print("\nReader/codec cancellation passed for these fixtures. TCP delivery does not prove device consumption or writer cancellation.")
