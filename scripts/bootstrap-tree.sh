#!/usr/bin/env bash

## Development bootstrap (step -1)
## {ROOT_DIR}/scripts/bootsrtap.sh

set -euo pipefail

ROOT_DIR="${1:-.}"

mkdir -p "${ROOT_DIR}"
mkdir -p "${ROOT_DIR}/docs"
mkdir -p "${ROOT_DIR}/scripts"
mkdir -p "${ROOT_DIR}/lisp"
mkdir -p "${ROOT_DIR}/tests/elisp"
mkdir -p "${ROOT_DIR}/tests/smoke"
mkdir -p "${ROOT_DIR}/docker"
mkdir -p "${ROOT_DIR}/data"

mkdir -p "${ROOT_DIR}/bridge/.cargo"

mkdir -p "${ROOT_DIR}/bridge/crates/mm-bridge/src/protocol"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-bridge/src/transport"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-bridge/src/server"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-bridge/src/app"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-bridge/src/config"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-bridge/src/status"

mkdir -p "${ROOT_DIR}/bridge/crates/mm-core/src/model"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-core/src/error"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-core/src/ids"

mkdir -p "${ROOT_DIR}/bridge/crates/mm-client/src/rest"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-client/src/ws"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-client/src/tls"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-client/src/auth"

mkdir -p "${ROOT_DIR}/bridge/crates/mm-store/src/cache"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-store/src/session"

mkdir -p "${ROOT_DIR}/bridge/crates/mm-web/src/routes"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-web/src/templates"
mkdir -p "${ROOT_DIR}/bridge/crates/mm-web/src/state"

touch "${ROOT_DIR}/README.md"
touch "${ROOT_DIR}/.gitignore"
touch "${ROOT_DIR}/.dockerignore"
touch "${ROOT_DIR}/compose.yaml"
touch "${ROOT_DIR}/docker/Dockerfile"
touch "${ROOT_DIR}/docker/entrypoint.sh"

touch "${ROOT_DIR}/docs/architecture.md"
touch "${ROOT_DIR}/docs/protocol.md"
touch "${ROOT_DIR}/docs/mvp-checklist.md"
touch "${ROOT_DIR}/docs/mattermost-10.9.3-notes.md"
touch "${ROOT_DIR}/docs/tls-self-signed.md"
touch "${ROOT_DIR}/docs/docker.md"

touch "${ROOT_DIR}/scripts/dev-check.sh"
touch "${ROOT_DIR}/scripts/docker-build.sh"
touch "${ROOT_DIR}/scripts/docker-up.sh"
touch "${ROOT_DIR}/scripts/docker-down.sh"
touch "${ROOT_DIR}/scripts/smoke-ping.sh"

touch "${ROOT_DIR}/bridge/Cargo.toml"
touch "${ROOT_DIR}/bridge/rust-toolchain.toml"
touch "${ROOT_DIR}/bridge/.cargo/config.toml"

touch "${ROOT_DIR}/bridge/crates/mm-bridge/Cargo.toml"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/main.rs"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/lib.rs"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/protocol/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/transport/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/server/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/app/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/config/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-bridge/src/status/mod.rs"

touch "${ROOT_DIR}/bridge/crates/mm-core/Cargo.toml"
touch "${ROOT_DIR}/bridge/crates/mm-core/src/lib.rs"
touch "${ROOT_DIR}/bridge/crates/mm-core/src/model/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-core/src/error/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-core/src/ids/mod.rs"

touch "${ROOT_DIR}/bridge/crates/mm-client/Cargo.toml"
touch "${ROOT_DIR}/bridge/crates/mm-client/src/lib.rs"
touch "${ROOT_DIR}/bridge/crates/mm-client/src/rest/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-client/src/ws/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-client/src/tls/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-client/src/auth/mod.rs"

touch "${ROOT_DIR}/bridge/crates/mm-store/Cargo.toml"
touch "${ROOT_DIR}/bridge/crates/mm-store/src/lib.rs"
touch "${ROOT_DIR}/bridge/crates/mm-store/src/cache/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-store/src/session/mod.rs"

touch "${ROOT_DIR}/bridge/crates/mm-web/Cargo.toml"
touch "${ROOT_DIR}/bridge/crates/mm-web/src/lib.rs"
touch "${ROOT_DIR}/bridge/crates/mm-web/src/routes/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-web/src/templates/mod.rs"
touch "${ROOT_DIR}/bridge/crates/mm-web/src/state/mod.rs"

touch "${ROOT_DIR}/lisp/mattermost.el"
touch "${ROOT_DIR}/lisp/mattermost-custom.el"
touch "${ROOT_DIR}/lisp/mattermost-process.el"
touch "${ROOT_DIR}/lisp/mattermost-protocol.el"
touch "${ROOT_DIR}/lisp/mattermost-server.el"
touch "${ROOT_DIR}/lisp/mattermost-model.el"
touch "${ROOT_DIR}/lisp/mattermost-buffer.el"
touch "${ROOT_DIR}/lisp/mattermost-dm.el"
touch "${ROOT_DIR}/lisp/mattermost-channel.el"
touch "${ROOT_DIR}/lisp/mattermost-thread.el"
touch "${ROOT_DIR}/lisp/mattermost-compose.el"
touch "${ROOT_DIR}/lisp/mattermost-faces.el"
touch "${ROOT_DIR}/lisp/mattermost-utils.el"

touch "${ROOT_DIR}/tests/elisp/mattermost-test.el"
touch "${ROOT_DIR}/tests/smoke/ping.jsonl"

chmod +x "${ROOT_DIR}/scripts/"*.sh
chmod +x "${ROOT_DIR}/docker/entrypoint.sh"

cat > "${ROOT_DIR}/README.md" <<'EOF'
# mattermost.el

Emacs client for corporate Mattermost, inspired by the UX ideas of `telega.el`.

Target Mattermost server version: **10.9.3**.

The project is split into:

- Emacs Lisp package: UI, buffers, commands, keybindings.
- Rust bridge: Mattermost REST/WebSocket integration, session cache, online updates.
- Docker Compose: primary way to run the bridge.
- Web status UI: exposed by the bridge container.

## Current development status

This repository is being built incrementally.

Do not implement large batches at once. Each step must be small, verifiable, and leave the project in a working state.

## Fixed decisions

- Bridge language: Rust.
- Main bridge runtime: Docker Compose.
- Emacs does not start Docker containers.
- Emacs connects to an already running bridge.
- Emacs ↔ bridge transport: TCP JSON Lines on localhost.
- Bridge exposes a web status UI.
- Mattermost server is the single source of truth.
- Emacs buffers are display-only projections.
- MVP message rendering: raw text, no Markdown rendering.
- MVP starts with direct messages to self.
- Channel messages come after self-DM flow works.
- Multiserver support is architecturally planned, but not implemented in MVP.
- Commands and identities must be server-scoped internally from the beginning.
- Direct messages must display server/team context to avoid confusion later.
- Threads/replies should be separate buffers by default, configurable via `use-package`.
- `g` in a Mattermost buffer means forced refresh of the buffer and connection health/recovery.
- Typing indicators are not part of MVP.
- Reactions are not part of MVP.
- Authentication MVP: login/password and saved session/PAT token.
- Token cache MVP: separate plain file inside bridge cache directory.
- Password may be read from `~/.authinfo`; no strong protection in MVP.
- Docker deployment must avoid leaking secrets through logs/environment dumps.
- Self-signed certificate support must be explicit through CA configuration, not global verification disablement in the bridge.

## Mattermost API notes

Mattermost REST API v4 is used for normal operations such as login, posting messages, channels, users, and files.

Mattermost WebSocket endpoint:

```text
/api/v4/websocket
```

WebSocket authentication can use standard API auth or an `authentication_challenge` action after connecting.

Authentication challenge shape:

```json
{
  "seq": 1,
  "action": "authentication_challenge",
  "data": {
    "token": "mattermosttokengoeshere"
  }
}
```

Successful response shape:

```json
{
  "status": "OK",
  "seq_reply": 1
}
```

After successful authentication, Mattermost sends a `hello` event.

Direct message channel creation endpoint:

```text
POST /api/v4/channels/direct
```

Request body is an array of two user IDs. For self-DM MVP we will use:

```json
["my_user_id", "my_user_id"]
```

This must be verified against the target corporate Mattermost 10.9.3 instance during implementation.

## Repository layout

```text
.
├── README.md
├── compose.yaml
├── docker/
│   ├── Dockerfile
│   └── entrypoint.sh
├── docs/
│   ├── architecture.md
│   ├── protocol.md
│   ├── mvp-checklist.md
│   ├── mattermost-10.9.3-notes.md
│   ├── tls-self-signed.md
│   └── docker.md
├── scripts/
│   ├── bootstrap-tree.sh
│   ├── dev-check.sh
│   ├── docker-build.sh
│   ├── docker-up.sh
│   ├── docker-down.sh
│   └── smoke-ping.sh
├── bridge/
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── crates/
│       ├── mm-bridge/
│       ├── mm-core/
│       ├── mm-client/
│       ├── mm-store/
│       └── mm-web/
├── lisp/
│   ├── mattermost.el
│   ├── mattermost-custom.el
│   ├── mattermost-process.el
│   ├── mattermost-protocol.el
│   ├── mattermost-server.el
│   ├── mattermost-model.el
│   ├── mattermost-buffer.el
│   ├── mattermost-dm.el
│   ├── mattermost-channel.el
│   ├── mattermost-thread.el
│   ├── mattermost-compose.el
│   ├── mattermost-faces.el
│   └── mattermost-utils.el
└── tests/
    ├── elisp/
    └── smoke/
```

## Planned bridge crates

### `mm-core`

Domain types and shared errors.

No network code.

Planned types:

- `ServerId`
- `TeamId`
- `ChannelId`
- `UserId`
- `PostId`
- normalized `Server`
- normalized `Team`
- normalized `Channel`
- normalized `User`
- normalized `Post`
- normalized errors

### `mm-client`

Mattermost REST and WebSocket client.

Responsibilities:

- login/password
- bearer token requests
- REST calls
- WebSocket connection
- WebSocket authentication challenge
- event parsing
- reconnect support
- TLS CA configuration

### `mm-store`

Local cache/session storage.

Responsibilities:

- session token file
- PAT file
- lightweight cache metadata
- server-scoped paths

MVP storage is plain text/json without encryption.

### `mm-bridge`

Main daemon.

Responsibilities:

- TCP JSONL server for Emacs
- request routing
- app state
- connection manager
- forced refresh handling
- event fanout to Emacs clients

### `mm-web`

Web status UI.

Responsibilities:

- `/health`
- `/status`
- simple HTML status page
- connection state
- last error
- Mattermost server version
- authenticated user summary, without secrets

## Emacs package design

MVP should eventually provide:

```elisp
(use-package mattermost
  :load-path "~/path/to/mattermost-el/lisp"
  :custom
  (mattermost-bridge-host "127.0.0.1")
  (mattermost-bridge-port 8765)
  (mattermost-thread-display 'separate-buffer)
  :bind
  (("C-c m c" . mattermost-connect)
   ("C-c m s" . mattermost-open-self-dm)
   ("C-c m g" . mattermost-refresh)))
```

Thread display option:

```elisp
mattermost-thread-display
```

Allowed values:

```text
separate-buffer
inline
```

MVP implements `separate-buffer` first. `inline` can be added later.

## Buffer naming rule

Even though MVP implements only one active server connection, names must already include server identity.

Planned examples:

```text
*Mattermost[work]/DM:self*
*Mattermost[work]/team:dev/channel:town-square*
*Mattermost[work]/team:dev/thread:post-id*
```

For direct messages with users later:

```text
*Mattermost[work]/team:dev/DM:alice*
*Mattermost[other]/team:platform/DM:alice*
```

This is required to avoid confusion between same usernames on different servers or teams.

## Online update rule

Open buffers must update online from bridge events.

`g` is not just rereading local data. It must:

1. ask bridge to check connection health;
2. reconnect WebSocket if needed;
3. refetch current buffer data from Mattermost REST API;
4. replace buffer contents from server state;
5. preserve user's unsent input where possible.

## MVP command set

Bridge methods for MVP:

```text
ping
status
connect
me
teams
list_channels
get_history
send_post
reply_post
mark_read
```

Additional internal/self-DM helper method planned:

```text
open_self_dm
```

This helper is needed because the first vertical slice starts from messages to self.

## MVP implementation checklist

### Phase 0 — repository bootstrap

- [x] Create repository tree script.
- [x] Create README with fixed decisions.
- [ ] Fill initial docs.
- [ ] Add `.gitignore`.
- [ ] Add `.dockerignore`.

### Phase 1 — minimal Rust bridge

- [ ] Add Cargo workspace.
- [ ] Add crate manifests.
- [ ] Add minimal `mm-core` types.
- [ ] Add JSONL protocol types.
- [ ] Add TCP listener.
- [ ] Implement `ping`.
- [ ] Implement `status`.
- [ ] Add smoke test for `ping`.

### Phase 2 — Docker and status web UI

- [ ] Add Dockerfile.
- [ ] Add compose.yaml.
- [ ] Add bridge config from environment.
- [ ] Add `/health`.
- [ ] Add `/status` JSON.
- [ ] Add simple HTML status page.
- [ ] Verify bridge runs separately from Emacs.

### Phase 3 — authentication

- [ ] Implement login/password against Mattermost 10.9.3.
- [ ] Save session token to cache file.
- [ ] Load saved session token.
- [ ] Support PAT token.
- [ ] Implement `connect`.
- [ ] Implement `me`.
- [ ] Ensure tokens are never logged.

### Phase 4 — self-DM vertical slice

- [ ] Implement `teams`.
- [ ] Resolve current user.
- [ ] Create/get self direct channel.
- [ ] Implement `open_self_dm`.
- [ ] Implement `get_history`.
- [ ] Implement `send_post`.
- [ ] Connect Emacs to bridge.
- [ ] Open self-DM buffer.
- [ ] Display raw messages.
- [ ] Send message from Emacs.
- [ ] Receive own message online.

### Phase 5 — WebSocket online updates

- [ ] Open Mattermost WebSocket.
- [ ] Authenticate with `authentication_challenge`.
- [ ] Handle `hello`.
- [ ] Handle `posted`.
- [ ] Handle `post_edited`.
- [ ] Handle `post_deleted`.
- [ ] Fan out normalized events to Emacs.
- [ ] Update open self-DM buffer online.
- [ ] Add reconnect loop.
- [ ] Implement forced refresh path for `g`.

### Phase 6 — replies/threads

- [ ] Implement `reply_post`.
- [ ] Add thread buffer.
- [ ] Add `mattermost-thread-display`.
- [ ] Default to separate thread buffers.
- [ ] Keep inline mode as future-compatible option.
- [ ] Show server/team context in thread buffer names.

### Phase 7 — channels

- [ ] Implement `list_channels`.
- [ ] Add channel list buffer.
- [ ] Open channel buffer.
- [ ] Load channel history.
- [ ] Send channel message.
- [ ] Receive channel events online.
- [ ] Implement `mark_read`.
- [ ] Forced refresh with `g`.

### Phase 8 — TLS/self-signed support

- [ ] Add explicit CA file configuration.
- [ ] Document Docker CA mount.
- [ ] Document Emacs CA settings separately.
- [ ] Do not disable certificate verification by default in Rust bridge.
- [ ] Add clear error message for TLS failures.

### Phase 9 — hardening

- [ ] Add structured errors.
- [ ] Add request timeouts.
- [ ] Add rate-limit handling.
- [ ] Add basic integration smoke tests.
- [ ] Add Emacs Lisp tests.
- [ ] Review logs for secret leaks.
- [ ] Review cache file permissions.
- [ ] Review multiserver-ready identifiers and naming.

## Development rules

- Work in small atomic steps.
- Every code answer must contain full files, not patches.
- Code files should stay under 300 lines.
- Split modules before they grow too large.
- Keep README checklist updated.
- Prefer simple implementations that can be verified immediately.
- Do not implement multiserver UI yet, but keep server-scoped IDs and state boundaries.
- Do not render Markdown in MVP.
- Do not add typing or reactions in MVP.
- Do not make Emacs responsible for Mattermost protocol details.
- Do not store authoritative chat state in Emacs buffers.

## Bootstrap

Run:

```bash
chmod +x scripts/bootstrap-tree.sh
./scripts/bootstrap-tree.sh .
```

The script creates directories and empty placeholder files. It also writes this README.

## Immediate next step

Next atomic step:

```text
Add initial ignore files and documentation skeletons.
```

After that:

```text
Add Rust workspace and minimal compiling bridge with ping/status over TCP JSONL.
```
EOF

cat > "${ROOT_DIR}/docs/mvp-checklist.md" <<'EOF'
# MVP checklist

This file mirrors the README checklist and can be used as a short working tracker.

## Done

- [x] Project decisions recorded.
- [x] Repository bootstrap script created.
- [x] README created.

## Next

- [ ] Initial ignore files.
- [ ] Initial docs.
- [ ] Rust workspace.
- [ ] Minimal TCP JSONL bridge.
- [ ] `ping`.
- [ ] `status`.
- [ ] Docker Compose.
- [ ] Web status page.
- [ ] Login/password.
- [ ] Token cache.
- [ ] `connect`.
- [ ] `me`.
- [ ] `teams`.
- [ ] Self-DM.
- [ ] History.
- [ ] Send post.
- [ ] Emacs self-DM buffer.
- [ ] WebSocket posted events.
- [ ] Forced refresh with `g`.
- [ ] Reply/thread buffers.
- [ ] Channels.
- [ ] Mark read.
- [ ] TLS CA configuration.
EOF

echo "Repository tree created at: ${ROOT_DIR}"
echo "Dirs: ${#DIRS[@]}, files: ${#FILES[@]}"
