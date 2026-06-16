import json
import os
import subprocess
import sys


DEFAULT_TIMEOUT_SECS = 2.0
DEFAULT_HERMESHIP_BIN = "__HERMESHIP_BIN__"


def handle(event_type, context):
    try:
        payload = {
            "provider": "hermes",
            "source": "gateway",
            "event": str(event_type or ""),
            "context": context if isinstance(context, dict) else {},
        }
        raw_payload = json.dumps(payload, ensure_ascii=False, separators=(",", ":"))
        result = subprocess.run(
            [_hermeship_binary(), "hermes", "hook", "--payload", "-"],
            input=raw_payload,
            text=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            timeout=_timeout_secs(),
            check=False,
        )
        if result.returncode != 0:
            _diagnose(
                "child-exit",
                f"hermeship exited with status {result.returncode}: {_tail(result.stderr)}",
            )
    except Exception as exc:
        _diagnose(type(exc).__name__, str(exc))
    return None


def _hermeship_binary():
    configured = os.environ.get("HERMESHIP_BIN", "").strip()
    return configured or DEFAULT_HERMESHIP_BIN


def _timeout_secs():
    raw = os.environ.get("HERMESHIP_HOOK_TIMEOUT_SECS", "").strip()
    if not raw:
        return DEFAULT_TIMEOUT_SECS
    try:
        timeout = float(raw)
    except ValueError:
        return DEFAULT_TIMEOUT_SECS
    return timeout if timeout > 0 else DEFAULT_TIMEOUT_SECS


def _diagnose(kind, detail):
    print(f"[hermeship] hook bridge skipped: {kind}: {_tail(detail)}", file=sys.stderr)


def _tail(value, limit=240):
    text = str(value or "").replace("\n", " ").strip()
    if len(text) <= limit:
        return text
    return text[-limit:]
