# emacs-mattermost

Emacs client for corporate Mattermost, inspired by the UX of telega.el.

Target Mattermost server version: **10.9.3**.

## Components

- Emacs Lisp package: UI, buffers, commands, keybindings.
- Rust bridge: Mattermost REST/WebSocket, session cache, online updates.
- Docker Compose: primary way to run the bridge.
- Web status UI: exposed by the bridge container.

## Fixed decisions

- Bridge language: Rust.
- Main bridge runtime: Docker Compose, started separately from Emacs.
- Emacs connects to an already running bridge over localhost TCP JSON Lines.
- Mattermost server is the single source of truth; Emacs buffers are display-only.
- MVP message rendering: raw text, no Markdown.
- MVP starts with direct messages to self, channels come later.
- Multiserver support is architecturally planned, not implemented in MVP.
- All identifiers and stored state are server-scoped from the beginning.
- DM buffers must show server/team context to avoid confusion.
- Threads/replies are separate buffers by default, configurable via use-package.
- `g` forces buffer refresh plus connection health check/recovery.
- Typing indicators and reactions are out of MVP scope.
- Auth MVP: login/password plus saved session token or PAT.
- Token cache: plain file per server under the data dir, private file mode.
- Password may be read from Emacs `~/.authinfo`; no strong protection in MVP.
- Secrets must never appear in logs, protocol responses, or the web status UI.
- TLS verification stays enabled; internal CA is configured explicitly.

## Build environment

Corporate network: no direct internet access, authenticated proxy.

- Cargo dependencies resolve through the Nexus mirror configured in
  `bridge/.cargo/config.toml`.
- Docker builder image: `nexus.local/rust:1.90-bookworm`.
- Docker runtime image: `nexus.local/ci/debian:bookworm-ca`.
- Host toolchain: Rust 1.94 (used to generate the lockfile).
- Workspace `rust-version`: 1.90.
- MSRV-aware resolution is enabled via
  `resolver.incompatible-rust-versions = "fallback"`, so cargo picks
  dependency versions compatible with Rust 1.90 automatically.
- `Cargo.lock` is committed and all builds use `--locked`.
- The corporate CA bundle is copied into the build context by
  `scripts/docker-prepare-ca.sh` so cargo trusts `nexus.local`.

Do not hand-pin transitive dependency versions. If a dependency needs
updating, run `cargo update`, then `scripts/dev-check.sh`, then the Docker
build. The resolver handles MSRV compatibility.

`bridge/rust-toolchain.toml` pins the host toolchain only. It is deliberately
not copied into the Docker image; the image uses its preinstalled toolchain.

## Repository layout

```text
.
├── README.md
├── compose.yaml
├── docker/
│   ├── Dockerfile
│   ├── entrypoint.sh
│   └── certs/            generated CA bundle for the build context
├── docs/
├── scripts/
├── bridge/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── rust-toolchain.toml
│   ├── .cargo/config.toml
│   └── crates/
│       ├── mm-core/      domain types, ids, errors (no network)
│       ├── mm-client/    Mattermost REST/WebSocket client
│       ├── mm-store/     token and server-config storage
│       ├── mm-web/       web status UI
│       └── mm-bridge/    daemon: TCP JSONL, routing, app state
├── lisp/
└── tests/
```

## Data directory

`MM_DATA_DIR` (default `/data` in Docker, `~/.local/share/emacs-mattermost`
on the host):

```text
<data_dir>/tokens/<server_id>.token    bearer token, mode 0600, atomic write
<data_dir>/servers/<server_id>.json    non-secret server config
```

`server_id` must match `[A-Za-z0-9_-]+`. This prevents path traversal and
keeps filenames predictable for future multiserver support.

## Ports

```text
127.0.0.1:8765   bridge TCP JSONL (Emacs connects here)
127.0.0.1:8766   web status UI (/ , /health , /status)
```

Both are bound to localhost only.

## Quick start

Start the bridge:

```bash
./scripts/docker-up.sh
```

Check it:

```bash
./scripts/smoke-ping.sh
./scripts/smoke-web.sh
```

Status page: http://127.0.0.1:8766/

Stop:

```bash
./scripts/docker-down.sh
```

Local development without Docker:

```bash
./scripts/dev-check.sh          # fmt check + cargo check/test --locked
cd bridge && cargo run -p mm-bridge
```

## Protocol

One JSON object per line over TCP. See `docs/protocol.md`.

Implemented methods:

```text
ping
status
```

Planned MVP methods:

```text
connect
me
teams
open_self_dm
list_channels
get_history
send_post
reply_post
mark_read
```

## Emacs package configuration

Add this section to your `init.el`:

```elisp
(use-package mattermost
  :load-path "~/path/to/emacs-mattermost/lisp" ;; Update path or remove after MELPA install
  :demand t
  :custom
  (mattermost-bridge-host "127.0.0.1")
  (mattermost-bridge-port 8765)
  (mattermost-server-url "https://mattermost.your-company.local")
  (mattermost-server-id "work")
  (mattermost-thread-display 'separate-buffer)
  (mattermost-download-dir "~/Downloads")
  :bind
  (("C-c M d" . mattermost-open-self-dm)
   ("C-c M D" . mattermost-dashboard)
   ("C-c M r" . mattermost-reset-state))
  :config
  ;; Recommended: use auth-source instead of plain text password
  ;; (setq mattermost-password (auth-source-pass-get 'secret "mattermost/work"))
  (message "Mattermost package loaded. Use C-c M d to open self-DM."))
```

Buffer naming keeps server identity visible even in single-server MVP:

```text
*Mattermost[work]/DM:self*
*Mattermost[work]/team:dev/channel:town-square*
*Mattermost[work]/team:dev/thread:post-id*
```

## Development rules

- Work in small atomic steps; each step must leave the project buildable.
- Provide full file contents, not patches.
- Keep code files under 300 lines; split modules before they grow.
- Prefer automated fixes (`cargo fmt`, MSRV-aware resolver) over manual edits.
- Commit `Cargo.lock` only after a fully green build.
- Keep this README and `docs/mvp-checklist.md` updated.
- Never log tokens, passwords, or auth headers.

## Progress

See `docs/mvp-checklist.md` for the detailed checklist.

Current state: phases 0-3a complete (storage foundation).
Next: phase 3b (REST client, auth, connect/me).

## Verification

To verify the bridge works against your corporate Mattermost instance:

1. Start the bridge: `./scripts/docker-up.sh`
2. Connect and get a token:
   ```bash
   ./scripts/mm-cli.sh '{"id":"1","type":"request","method":"connect","params":{"server_id":"work","url":"https://mm.local","login_id":"user","password":"pass"}}'
   ```
3. Open self-DM channel:
   ```bash
   ./scripts/mm-cli.sh '{"id":"2","type":"request","method":"open_self_dm","params":{"server_id":"work"}}'
   ```
4. Send a message to yourself:
   ```bash
   ./scripts/mm-cli.sh '{"id":"3","type":"request","method":"send_post","params":{"server_id":"work","channel_id":"<ID_FROM_STEP_3>","message":"Test"}}'
   ```
4. Check history:
   ```bash
   ./scripts/mm-cli.sh '{"id":"4","type":"request","method":"get_history","params":{"server_id":"work","channel_id":"<ID_FROM_STEP_3>","limit":1}}'
   ```

## MELPA Readiness Checklist

- [x] Standard package header in `lisp/mattermost.el` (`;; Version`, `;; Package-Requires`, `;; License`).
- [x] Missing `lisp/mattermost-files.el` created and required.
- [x] Dockerfile uses public images by default (reproducible builds), removable `# syntax` directive to avoid DNS timeouts.
- [ ] `LICENSE` file (GPLv3+) added to the repository root.
- [ ] `Makefile` added with `compile` and `lint` targets for CI.
- [ ] CI pipeline configured to run `make lint` and `make compile` on push.
- [ ] Recipe submitted to the `melpa/melpa` repository.
