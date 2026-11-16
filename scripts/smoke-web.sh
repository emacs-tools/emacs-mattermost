#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${MM_WEB_URL:-http://127.0.0.1:8766}"

python3 - "$BASE_URL" <<'PY'
import json
import sys
import urllib.request

# Ignore system proxy settings: we talk to localhost directly.
proxy_handler = urllib.request.ProxyHandler({})
opener = urllib.request.build_opener(proxy_handler)
urllib.request.install_opener(opener)

base_url = sys.argv[1].rstrip("/")

def get_json(path):
    with urllib.request.urlopen(base_url + path, timeout=5) as response:
        body = response.read().decode("utf-8")
        return json.loads(body)

health = get_json("/health")
print(json.dumps(health, ensure_ascii=False))
if health.get("ok") is not True:
    raise SystemExit("/health is not ok")

status = get_json("/status")
print(json.dumps(status, ensure_ascii=False))
if status.get("bridge", {}).get("state") != "running":
    raise SystemExit("/status bridge.state is not running")

with urllib.request.urlopen(base_url + "/", timeout=5) as response:
    html = response.read().decode("utf-8")
    if "emacs-mattermost bridge" not in html:
        raise SystemExit("index page does not contain expected title")
    print("index html ok")
PY
