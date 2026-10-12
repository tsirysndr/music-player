"""Check evidence integrity/positive controls, not product compatibility."""
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
summary = json.loads((root / "summary.json").read_text())
assert len(summary["cases"]) == 7, "expected 7 experiment cases before deduplication"
cases = {case["case"]: case for case in summary["cases"]}
expected = {"finite_no_range", "chunked", "late_headers", "body_stall", "header_required", "fake_audio", "short_eof"}
assert cases.keys() == expected, "missing experiment evidence"
for name in ("finite_no_range", "chunked", "short_eof"):
    assert cases[name]["nonzero_pcm_bytes"] > 10000, f"{name}: positive decode control failed"
    assert cases[name]["drop_joined"], f"{name}: positive control did not exit"
for name in ("header_required", "fake_audio"):
    assert cases[name]["nonzero_pcm_bytes"] == 0, f"{name}: invalid media produced nonzero audio"
for name, case in cases.items():
    evidence = json.loads((root / f"{name}.json").read_text())
    assert evidence["summary"] == case, f"{name}: per-case evidence summary diverges from report"
    assert case["request_count"] == len(evidence["requests"]), f"{name}: request count inconsistent with evidence"
    assert case["nonzero_pcm_bytes"] == sum(e["nonzero_bytes"] for e in evidence["pcm"]), f"{name}: PCM totals inconsistent with evidence"
    assert case["drop_joined"] == any(e["event"] == "drop_joined" for e in evidence["engine_events"]), f"{name}: drop_joined inconsistent with engine events"
    assert evidence["requests"], f"{name}: missing HTTP request"
    assert any(e["event"] == "stop_called" for e in evidence["engine_events"]), f"{name}: stop never issued"
    assert case["internal_cancel_gate"] != "Pass", "public API cannot establish reader join"
    if case["watchdog_killed"]:
        assert case["exit_code"] != 0 and not case["drop_joined"], f"{name}: watchdog kill must be a nonzero exit without drop join"
        assert case["internal_cancel_gate"] == "Fail", f"{name}: watchdog kill must record cancellation failure"
    else:
        assert case["exit_code"] == 0 and case["drop_joined"], f"{name}: clean run must exit 0 with drop joined"
print("Evidence complete; positive decode/error controls passed. Product M1 gate remains unverified/failed.")
print("| Scenario | Nonzero PCM bytes | Watchdog killed | Drop joined | Internal cancellation |")
print("| --- | ---: | --- | --- | --- |")
for name, case in cases.items():
    print(f"| {name} | {case["nonzero_pcm_bytes"]} | {case["watchdog_killed"]} | {case["drop_joined"]} | {case["internal_cancel_gate"]} |")
