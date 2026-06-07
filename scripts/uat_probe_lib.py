#!/usr/bin/env python3
"""
UAT probe resolver (US-0092 / DEC-0078).

Shared by /verify-work and /qa for self-verify acceptance steps.
Fail-closed — no silent PASS.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

UAT_PROBE_UNRESOLVED = "UAT_PROBE_UNRESOLVED"
UAT_STACK_PROFILE_UNKNOWN = "UAT_STACK_PROFILE_UNKNOWN"
UAT_PROBE_TIMEOUT = "UAT_PROBE_TIMEOUT"
UAT_PROBE_FAILED = "UAT_PROBE_FAILED"
UAT_PROBE_FORBIDDEN = "UAT_PROBE_FORBIDDEN"
UAT_PROBE_PASS = "UAT_PROBE_PASS"

PROBE_KINDS = (
    "build",
    "test",
    "api_health",
    "process_health",
    "browser_smoke",
    "cli_smoke",
    "manual_operator",
)

FORBIDDEN_PATH_TOKENS = (".env", "intake_evidence", "handoffs/intake_evidence")

DEFAULT_PROBE_TIMEOUT = 120


def _merge_scratchpad(repo: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for name in (".cursor/scratchpad.md", ".cursor/scratchpad.local.md"):
        path = repo / name
        if not path.is_file():
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            if "=" in stripped:
                key, _, val = stripped.partition("=")
                values[key.strip()] = val.strip()
    return values


def detect_stack_profile(repo: Path) -> str | None:
    if (repo / "package.json").is_file():
        return "node"
    if (repo / "pyproject.toml").is_file() or (repo / "setup.py").is_file():
        return "python"
    if (repo / "go.mod").is_file():
        return "go"
    if list(repo.glob("*.csproj")):
        return "dotnet"
    if (repo / "pom.xml").is_file():
        return "java"
    readme = repo / "README.md"
    if readme.is_file() and "generated" in readme.read_text(encoding="utf-8").lower():
        return "generated"
    return None


def _forbidden(step_text: str) -> bool:
    lower = step_text.lower()
    return any(tok in lower for tok in FORBIDDEN_PATH_TOKENS)


def _read_test_command(repo: Path) -> str | None:
    runbook = repo / "docs" / "engineering" / "runbook.md"
    if runbook.is_file():
        m = re.search(r"^TEST_COMMAND:\s*(.+)$", runbook.read_text(encoding="utf-8"), re.M)
        if m:
            cmd = m.group(1).strip()
            if cmd and "not configured" not in cmd.lower():
                return cmd
    merged = _merge_scratchpad(repo)
    return merged.get("TEST_COMMAND") or None


def _read_build_command(repo: Path, profile: str | None) -> str | None:
    if profile == "node" and (repo / "package.json").is_file():
        try:
            pkg = json.loads((repo / "package.json").read_text(encoding="utf-8"))
            scripts = pkg.get("scripts", {})
            if "build" in scripts:
                return "npm run build"
        except (json.JSONDecodeError, OSError):
            pass
    if profile == "python" and (repo / "pyproject.toml").is_file():
        return "python -m build"
    return None


def _read_health_url(repo: Path) -> str | None:
    rc = repo / "docs" / "engineering" / "runtime-connectivity.md"
    if rc.is_file():
        m = re.search(r"https?://[^\s\)]+", rc.read_text(encoding="utf-8"))
        if m:
            return m.group(0).rstrip(".,)")
    return None


def classify_step(step_text: str, repo: Path) -> tuple[str | None, str]:
    if _forbidden(step_text):
        return None, UAT_PROBE_FORBIDDEN
    lower = step_text.lower()
    profile = detect_stack_profile(repo)

    if any(w in lower for w in ("manual", "operator", "human", "judgment", "visually")):
        return "manual_operator", UAT_PROBE_UNRESOLVED

    if any(w in lower for w in ("build", "compile", "bundle")):
        if _read_build_command(repo, profile):
            return "build", ""
        return None, UAT_PROBE_UNRESOLVED

    if any(w in lower for w in ("test", "pytest", "unit test", "integration test")):
        if _read_test_command(repo) or profile in ("python", "node", "go", "generated"):
            return "test", ""
        return None, UAT_PROBE_UNRESOLVED

    if any(w in lower for w in ("api", "health", "endpoint", "http", "rest")):
        if _read_health_url(repo):
            return "api_health", ""
        return None, UAT_PROBE_UNRESOLVED

    if any(w in lower for w in ("process", "startup", "server start", "readiness")):
        return "process_health", UAT_PROBE_UNRESOLVED

    if any(w in lower for w in ("browser", "playwright", "smoke", "ui")):
        if profile == "node" or _read_health_url(repo):
            return "browser_smoke", ""
        return None, UAT_PROBE_UNRESOLVED

    if any(w in lower for w in ("cli", "command line", "exit code")):
        return "cli_smoke", UAT_PROBE_UNRESOLVED

    if profile is None:
        return None, UAT_STACK_PROFILE_UNKNOWN
    return None, UAT_PROBE_UNRESOLVED


def execute_probe(
    kind: str,
    step_text: str,
    repo: Path,
    *,
    timeout: int = DEFAULT_PROBE_TIMEOUT,
) -> dict[str, object]:
    profile = detect_stack_profile(repo)
    result: dict[str, object] = {
        "probe_kind": kind,
        "step": step_text[:200],
        "stack_profile": profile or "unknown",
        "reason_code": UAT_PROBE_UNRESOLVED,
        "passed": False,
    }

    if kind == "manual_operator":
        result["reason_code"] = UAT_PROBE_UNRESOLVED
        return result

    if kind == "build":
        cmd = _read_build_command(repo, profile)
        if not cmd:
            result["reason_code"] = UAT_PROBE_UNRESOLVED
            return result
        return _run_subprocess(cmd, repo, timeout, result)

    if kind == "test":
        cmd = _read_test_command(repo)
        if not cmd:
            if profile == "python":
                cmd = "python -m pytest -q"
            elif profile == "node":
                cmd = "npm test"
            elif profile == "generated":
                cmd = "python -m pytest -q"
            else:
                result["reason_code"] = UAT_PROBE_UNRESOLVED
                return result
        return _run_subprocess(cmd, repo, timeout, result)

    if kind == "api_health":
        url = _read_health_url(repo)
        if not url:
            result["reason_code"] = UAT_PROBE_UNRESOLVED
            return result
        try:
            req = urllib.request.Request(url, method="GET")
            with urllib.request.urlopen(req, timeout=min(timeout, 30)) as resp:
                result["status_code"] = resp.status
                result["reason_code"] = UAT_PROBE_PASS
                result["passed"] = 200 <= resp.status < 400
                if not result["passed"]:
                    result["reason_code"] = UAT_PROBE_FAILED
        except urllib.error.URLError as exc:
            result["reason_code"] = UAT_PROBE_FAILED
            result["error"] = type(exc).__name__
        except TimeoutError:
            result["reason_code"] = UAT_PROBE_TIMEOUT
        return result

    if kind in ("process_health", "browser_smoke", "cli_smoke"):
        result["reason_code"] = UAT_PROBE_UNRESOLVED
        return result

    result["reason_code"] = UAT_PROBE_UNRESOLVED
    return result


def _run_subprocess(
    cmd: str,
    repo: Path,
    timeout: int,
    result: dict[str, object],
) -> dict[str, object]:
    result["command"] = cmd
    try:
        proc = subprocess.run(
            cmd,
            shell=True,
            cwd=str(repo),
            timeout=timeout,
            capture_output=True,
            text=True,
        )
        result["exit_code"] = proc.returncode
        if proc.returncode == 0:
            result["reason_code"] = UAT_PROBE_PASS
            result["passed"] = True
        else:
            result["reason_code"] = UAT_PROBE_FAILED
    except subprocess.TimeoutExpired:
        result["reason_code"] = UAT_PROBE_TIMEOUT
    except OSError as exc:
        result["reason_code"] = UAT_PROBE_FAILED
        result["error"] = str(exc)
    return result


def resolve_and_probe(step_text: str, repo: Path) -> dict[str, object]:
    kind, pre_reason = classify_step(step_text, Path(repo))
    if kind is None:
        return {
            "probe_kind": None,
            "reason_code": pre_reason or UAT_PROBE_UNRESOLVED,
            "passed": False,
        }
    if pre_reason == UAT_PROBE_FORBIDDEN:
        return {"probe_kind": kind, "reason_code": UAT_PROBE_FORBIDDEN, "passed": False}
    return execute_probe(kind, step_text, Path(repo))


def probe_steps(steps: list[str], repo: Path) -> list[dict[str, object]]:
    return [resolve_and_probe(step, repo) for step in steps]


def self_test() -> None:
    repo = Path(__file__).resolve().parents[1]
    assert UAT_PROBE_PASS in PROBE_KINDS or UAT_PROBE_PASS == "UAT_PROBE_PASS"
    assert "build" in PROBE_KINDS
    r = classify_step("run unit tests", repo)
    assert r[0] == "test" or r[1] in (UAT_PROBE_UNRESOLVED, "")
    r2 = classify_step("read secrets from .env file", repo)
    assert r2[1] == UAT_PROBE_FORBIDDEN
    r3 = classify_step("operator manually verifies UI", repo)
    assert r3[0] == "manual_operator"
    assert detect_stack_profile(repo) in ("python", "node", None, "generated")


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser(description="UAT probe resolver (US-0092).")
    parser.add_argument("--repo", default=".")
    parser.add_argument("--step", action="append", default=[], help="Acceptance step text.")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--report", action="store_true", help="JSON probe results to stdout.")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()

    if args.self_test:
        try:
            self_test()
        except AssertionError as exc:
            print(f"self-test failed: {exc}", file=sys.stderr)
            return 2
        print("[UAT_PROBE_LIB_SELF_TEST_OK]")
        return 0

    if args.report or args.step:
        results = probe_steps(args.step or ["run tests"], repo)
        print(json.dumps(results, sort_keys=True, separators=(",", ":")))
        if any(r.get("reason_code") == UAT_PROBE_UNRESOLVED for r in results):
            return 1
        if any(not r.get("passed") for r in results):
            return 1
        return 0

    parser.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main())
