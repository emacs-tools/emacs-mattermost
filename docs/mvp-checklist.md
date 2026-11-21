# MVP Checklist

## Fixed decisions

- [x] Emacs Lisp UI.
- [x] Rust bridge.
- [x] Docker Compose as primary bridge runtime.
- [x] Emacs does not start Docker.
- [x] TCP JSONL transport.
- [x] Web status UI.
- [x] Mattermost 10.9.3 target.
- [x] Mattermost server is source of truth.
- [x] Emacs buffers are display-only.
- [x] MVP starts with self-DM.
- [x] Raw message text, no Markdown rendering.
- [x] Threads separate buffers by default, configurable.
- [x] g means forced refresh and connection recovery.
- [x] Multiserver architecture planned, not implemented in MVP.
- [x] Typing not in MVP.
- [x] Reactions not in MVP.
- [x] Login/password and session/PAT token only.
- [x] Plain token file per server, private mode.
- [x] Explicit CA configuration, verification stays enabled.
- [x] Automated dependency resolution, no manual version pins.

## Phase 0 — repository bootstrap

- [x] Repository tree script.
- [x] README.
- [x] .gitignore / .dockerignore.
- [x] architecture / protocol / docker / tls / mattermost notes docs.
- [x] MVP checklist.

## Phase 1 — minimal Rust bridge

- [x] Cargo workspace, five crates.
- [x] mm-core: ids, errors, models.
- [x] JSONL protocol parse/validate/encode.
- [x] TCP listener, line-based handling.
- [x] ping, status, unknown_method, invalid_json.
- [x] Unit tests.
- [x] dev-check.sh, smoke-ping.sh.

## Phase 2 — Docker and web status UI

- [x] mm-web with axum.
- [x] /health, /status, HTML index.
- [x] Shared BridgeStatus between TCP and web.
- [x] Config from environment (listen addresses).
- [x] Dockerfile with Nexus builder and internal CA.
- [x] compose.yaml, docker helper scripts.
- [x] Non-root runtime user.
- [x] smoke-web.sh with proxy bypass.
- [x] MSRV-aware Cargo.lock, committed after green build.
- [x] Local bridge + web smoke verified.
- [x] Docker Compose bridge + web smoke verified.

## Phase 3a — storage foundation (no network)

- [x] mm-core: User model.
- [x] mm-core: ServerConfig model.
- [x] mm-core: ServerSummary.has_token.
- [x] mm-store: TokenStore (load/save/delete/list).
- [x] Atomic token writes via temp file + rename.
- [x] Private file mode 0600 on Unix.
- [x] server_id validation against path traversal.
- [x] mm-store: ServerStore for non-secret config.
- [x] MM_DATA_DIR config with host default.
- [x] App state carries data_dir.
- [x] status reads persisted stores, never exposes secrets.
- [x] Unit tests for stores and status.
- [x] Verify cargo test locally.
- [x] Verify status shows a stored server without leaking the token.

## Phase 3b — authentication (network)

- [x] mm-client: HTTP client with explicit CA support.
- [x] login/password against Mattermost 10.9.3, read Token header.
- [x] Save session token via TokenStore.
- [x] Load saved token on connect.
- [x] Support PAT token.
- [x] Implement connect.
- [x] Implement me.
- [x] Map Mattermost errors to normalized error codes.
- [x] Ensure tokens/passwords are never logged.

## Phase 4a — self-DM backend vertical slice

- [x] Implement teams.
- [x] Resolve current user.
- [x] Create/get self direct channel.
- [x] Implement open_self_dm.
- [x] Implement get_history with ascending post order.
- [x] Implement send_post.
- [x] Bridge methods verified via smoke tests.
- [x] Post model includes file_ids.

## Phase 4b — Emacs UI

- [x] Connect Emacs to bridge over TCP.
- [x] Interactive connection prompt (password in memory only).
- [x] Dashboard buffer with channel list (`mattermost-dashboard-mode`).
- [x] Open channel buffer from dashboard.
- [x] Display raw message history in channel buffer.
- [x] Send message from channel buffer (`C-c C-s`).
- [x] Refresh channel history (`g`).
- [x] Reset connection state (`mattermost-reset-state`).
- [ ] Receive own message online (WebSocket fan-out).
- [ ] Resolve DM channel display names (currently shows empty).

## Phase 5 — WebSocket online updates

- [ ] Open Mattermost WebSocket.
- [ ] Authenticate with authentication_challenge.
- [ ] Handle hello.
- [ ] Handle posted.
- [ ] Handle post_edited.
- [ ] Handle post_deleted.
- [ ] Fan out normalized events to Emacs.
- [ ] Update open self-DM buffer online.
- [ ] Add reconnect loop.
- [ ] Implement forced refresh path for g.

## Phase 6 — replies/threads

- [ ] Implement reply_post.
- [ ] Thread buffer.
- [ ] mattermost-thread-display option.
- [ ] Default separate-buffer.
- [ ] Keep inline mode future-compatible.
- [ ] Server/team context in thread buffer names.

## Phase 7 — channels

- [x] Implement list_channels (Rust backend).
- [x] Channel list buffer (Emacs UI).
- [x] Open channel buffer, load history, send message.
- [ ] Channel events online (WebSocket fan-out).
- [ ] Implement mark_read.
- [ ] Resolve DM channel display names.
- [ ] Forced refresh with g (basic refresh works, full recovery pending).

## Phase 8 — TLS/self-signed support

- [ ] MM_CA_FILE configuration for bridge to Mattermost.
- [ ] Same CA policy for REST and WebSocket.
- [ ] Document Docker CA mount.
- [ ] Clear tls_error messages.

## Phase 9 — hardening

- [ ] Request timeouts.
- [ ] Rate-limit handling.
- [ ] Integration smoke tests.
- [ ] Emacs Lisp tests.
- [ ] Log review for secret leaks.
- [ ] Cache permission review.
- [ ] Multiserver-ready naming review.

## Next atomic step

Phase 5: WebSocket client implementation in mm-client.
