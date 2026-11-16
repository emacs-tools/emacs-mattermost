# MVP Checklist

## Fixed decisions

- [x] Emacs Lisp UI.
- [x] Rust bridge.
- [x] Docker Compose as primary bridge runtime.
- [x] Emacs does not start Docker.
- [x] TCP JSONL transport.
- [x] Web status UI planned.
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
- [x] Plain token cache file for MVP.
- [x] Explicit CA configuration for self-signed/internal CA.

## Phase 0 — repository bootstrap

- [x] Create repository tree script.
- [x] Create README.
- [x] Create .gitignore.
- [x] Create .dockerignore.
- [x] Create architecture doc.
- [x] Create protocol doc.
- [x] Create Docker doc.
- [x] Create TLS doc.
- [x] Create Mattermost 10.9.3 notes.
- [x] Create MVP checklist.

## Phase 1 — minimal Rust bridge

- [x] Add Cargo workspace.
- [x] Add crate manifests.
- [x] Add minimal mm-core types.
- [x] Add JSONL protocol types.
- [x] Add TCP listener.
- [x] Implement ping.
- [x] Implement status.
- [x] Add smoke test for ping/status.

## Phase 2 — Docker and status web UI

- [x] Add Dockerfile.
- [x] Add compose.yaml.
- [x] Add bridge config from environment.
- [x] Add /health.
- [x] Add /status JSON.
- [x] Add simple HTML status page.
- [x] Add Docker helper scripts.
- [x] Add web smoke script.
- [ ] Verify local bridge + web smoke.
- [ ] Verify Docker Compose bridge + web smoke.

## Phase 3 — authentication

- [ ] Implement login/password against Mattermost 10.9.3.
- [ ] Save session token to cache file.
- [ ] Load saved session token.
- [ ] Support PAT token.
- [ ] Implement connect.
- [ ] Implement me.
- [ ] Ensure tokens are never logged.

## Phase 4 — self-DM vertical slice

- [ ] Implement teams.
- [ ] Resolve current user.
- [ ] Create/get self direct channel.
- [ ] Implement open_self_dm.
- [ ] Implement get_history.
- [ ] Implement send_post.
- [ ] Connect Emacs to bridge.
- [ ] Open self-DM buffer.
- [ ] Display raw messages.
- [ ] Send message from Emacs.
- [ ] Receive own message online.

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
- [ ] Add thread buffer.
- [ ] Add mattermost-thread-display.
- [ ] Default to separate thread buffers.
- [ ] Keep inline mode as future-compatible option.
- [ ] Show server/team context in thread buffer names.

## Phase 7 — channels

- [ ] Implement list_channels.
- [ ] Add channel list buffer.
- [ ] Open channel buffer.
- [ ] Load channel history.
- [ ] Send channel message.
- [ ] Receive channel events online.
- [ ] Implement mark_read.
- [ ] Forced refresh with g.

## Phase 8 — TLS/self-signed support

- [ ] Add explicit CA file configuration.
- [ ] Document Docker CA mount.
- [ ] Document Emacs CA settings separately.
- [ ] Do not disable certificate verification by default in Rust bridge.
- [ ] Add clear error message for TLS failures.

## Phase 9 — hardening

- [ ] Add structured errors.
- [ ] Add request timeouts.
- [ ] Add rate-limit handling.
- [ ] Add basic integration smoke tests.
- [ ] Add Emacs Lisp tests.
- [ ] Review logs for secret leaks.
- [ ] Review cache file permissions.
- [ ] Review multiserver-ready identifiers and naming.

## Next atomic step

Verify Phase 2 locally and in Docker Compose.

After that:

Implement authentication foundation: connect, me, login/password, token cache.
