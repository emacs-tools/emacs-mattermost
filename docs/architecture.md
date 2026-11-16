# Architecture

## Goal

Build an Emacs client for corporate Mattermost, inspired by the UX of telega.el.

Target Mattermost server version: 10.9.3.

The project is intentionally split into a thin Emacs UI and a long-lived Rust bridge.

## High-level components

Emacs Lisp package:

- user commands;
- buffers;
- keybindings;
- faces;
- message composition;
- display-only projections of server state.

Rust bridge:

- TCP JSON Lines server for Emacs;
- Mattermost REST API client;
- Mattermost WebSocket client;
- authentication/session handling;
- token cache;
- online event fanout;
- forced refresh/reconnect support;
- web status UI.

Docker Compose:

- primary way to run the bridge;
- bridge is started separately from Emacs;
- Emacs only connects to an already running bridge.

## Source of truth

Mattermost server is the single source of truth.

The bridge may keep temporary operational state and cache files.

Emacs buffers are not authoritative storage. They are projections that can be rebuilt from the server.

## Transport

Emacs connects to the bridge over localhost TCP.

Protocol format:

- one JSON object per line;
- UTF-8;
- request/response correlation by id;
- asynchronous events from bridge to Emacs.

Default planned address:

    127.0.0.1:8765

Logs must not be mixed into the protocol stream.

## Server-scoped model

MVP implements one active Mattermost connection, but the architecture must be multiserver-ready.

All domain objects and requests must be scoped by server identity from the beginning.

Planned identifiers:

- ServerId;
- TeamId;
- ChannelId;
- UserId;
- PostId.

Direct messages must eventually show server/team context in buffer names and UI labels.

Examples:

    *Mattermost[work]/DM:self*
    *Mattermost[work]/team:dev/DM:alice*
    *Mattermost[other]/team:platform/DM:alice*

This avoids confusion between identical usernames on different servers or teams.

## MVP vertical slice

The first working flow is self-DM:

1. start bridge with Docker Compose;
2. connect Emacs to bridge;
3. authenticate to Mattermost;
4. resolve current user;
5. open/create direct channel to self;
6. load history;
7. display raw messages;
8. send message;
9. receive online update through WebSocket;
10. force refresh with g.

Channel UI comes after self-DM works.

## Online updates

Open buffers must update online from bridge events.

The bridge owns WebSocket connection management.

Emacs should not talk directly to Mattermost REST or WebSocket APIs.

## Forced refresh with g

In Mattermost buffers, g means:

1. ask bridge to check connection health;
2. reconnect WebSocket if needed;
3. refetch current buffer data from Mattermost REST API;
4. replace buffer contents from server state;
5. preserve unsent input where possible.

g is a recovery action, not just a local redraw.

## Threads and replies

Replies/threads should be separate buffers by default.

This must be configurable from use-package.

Planned option:

    mattermost-thread-display

Allowed values:

    separate-buffer
    inline

MVP implements separate-buffer first.

inline is kept as a future-compatible option.

## Message rendering

MVP displays message text as-is.

No Markdown rendering in MVP.

Later rendering must preserve raw message text for editing and server synchronization.

## Authentication

MVP supports:

- login/password;
- saved session token;
- PAT token.

Token cache is a separate plain file for MVP.

Password may be read from ~/.authinfo.

Strong secret protection is postponed, but logs must never print tokens or passwords.

Docker deployment must avoid leaking secrets through logs or environment dumps.

## TLS and self-signed certificates

Corporate Mattermost may use self-signed or internal CA certificates.

Bridge TLS policy:

- do not disable certificate verification by default;
- support explicit CA file configuration;
- document Docker CA mount;
- produce clear TLS error messages.

Emacs-side TLS settings are separate from bridge TLS settings.

User currently has Emacs workaround settings:

    (setq gnutls-verify-error nil)
    (setq tls-checksig-warn-clock-skew t)
    (setq gnutls-trustfiles '("/etc/ssl/certs/ca-certificates.crt"))

These settings do not affect Rust bridge TLS validation.

## Crate boundaries

mm-core:

- domain types;
- identifiers;
- normalized models;
- shared errors;
- no network code.

mm-client:

- Mattermost REST client;
- Mattermost WebSocket client;
- authentication calls;
- TLS configuration;
- reconnect support.

mm-store:

- session token cache;
- PAT cache;
- lightweight metadata cache;
- server-scoped storage paths.

mm-bridge:

- main daemon;
- TCP JSONL server;
- request routing;
- app state;
- connection manager;
- event fanout;
- forced refresh handling.

mm-web:

- web status UI;
- health endpoint;
- status endpoint;
- simple HTML page;
- no secrets in responses.

## Emacs module boundaries

mattermost.el:

- entry point;
- autoloaded commands;
- high-level orchestration.

mattermost-custom.el:

- defgroup;
- defcustom options.

mattermost-process.el:

- TCP process lifecycle;
- reconnect to bridge;
- process filter/sentinel.

mattermost-protocol.el:

- JSONL encoding/decoding;
- request ids;
- pending callbacks;
- event dispatch.

mattermost-server.el:

- server connection model;
- server-scoped command routing.

mattermost-model.el:

- local display models;
- server/team/channel/user/post structs.

mattermost-buffer.el:

- common buffer behavior;
- refresh command;
- buffer-local state.

mattermost-dm.el:

- self-DM flow;
- direct message buffers.

mattermost-channel.el:

- channel list;
- channel buffers.

mattermost-thread.el:

- thread buffers;
- reply flow.

mattermost-compose.el:

- message input;
- send keybindings.

mattermost-faces.el:

- faces.

mattermost-utils.el:

- small shared helpers.

## Non-goals for MVP

- full multiserver UI;
- typing indicators;
- reactions;
- Markdown rendering;
- file upload/download;
- search;
- offline-first storage;
- admin functions;
- starting Docker from Emacs.
