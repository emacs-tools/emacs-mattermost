#!/usr/bin/env bash
set -euo pipefail

HOST="${MM_BRIDGE_HOST:-127.0.0.1}"
PORT="${MM_BRIDGE_PORT:-8765}"

python3 - "$HOST" "$PORT" <<'PY'
import json
import socket
import sys

host = sys.argv[1]
port = int(sys.argv[2])

requests = [
    {"id": "smoke-ping", "type": "request", "method": "ping", "params": {}},
    {"id": "smoke-status", "type": "request", "method": "status", "params": {}},
]

with socket.create_connection((host, port), timeout=5) as sock:
    file = sock.makefile("rwb")
    for request in requests:
        file.write((json.dumps(request) + "\n").encode("utf-8"))
        file.flush()
        line = file.readline()
        if not line:
            raise SystemExit("empty response")
        response = json.loads(line.decode("utf-8"))
        print(json.dumps(response, ensure_ascii=False))
        if response.get("id") != request["id"]:
            raise SystemExit("response id mismatch")
        if response.get("ok") is not True:
            raise SystemExit("response is not ok")
PY
