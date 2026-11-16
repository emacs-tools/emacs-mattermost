# Emacs ↔ Bridge Protocol

## Transport

Localhost TCP.

Default planned address:

    127.0.0.1:8765

Framing:

- UTF-8;
- one JSON object per line;
- newline delimiter;
- no pretty-printed multi-line JSON;
- no logs in protocol stream.

## Message kinds

Request from Emacs to bridge:

    {
      "id": "req-1",
      "type": "request",
      "method": "ping",
      "params": {}
    }

Response from bridge to Emacs:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "pong": true
      }
    }

Error response:

    {
      "id": "req-1",
      "type": "response",
      "ok": false,
      "error": {
        "code": "invalid_request",
        "message": "Missing method",
        "retryable": false
      }
    }

Event from bridge to Emacs:

    {
      "type": "event",
      "event": "post_created",
      "server_id": "work",
      "data": {}
    }

## Request rules

Every request has:

- id: string;
- type: "request";
- method: string;
- params: object.

The bridge must respond to every request exactly once.

The bridge must preserve request id in the response.

Unknown methods return invalid_request or unknown_method error.

Malformed JSON returns invalid_json error.

## Response rules

Every response has:

- id: same as request id;
- type: "response";
- ok: boolean;
- result: object when ok is true;
- error: object when ok is false.

Error object:

- code: stable machine-readable string;
- message: human-readable string;
- retryable: boolean.

## Event rules

Events have no request id.

Events are asynchronous.

Events must include server_id once multiserver-ready routing is active.

MVP may use a configured default server id such as "default".

Events must not contain tokens, passwords, or raw Authorization headers.

## MVP methods

Required MVP methods:

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

Additional helper for first vertical slice:

    open_self_dm

## ping

Request:

    {
      "id": "req-1",
      "type": "request",
      "method": "ping",
      "params": {}
    }

Response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "pong": true
      }
    }

Purpose:

- verify TCP connection;
- verify JSONL framing;
- verify request/response correlation.

## status

Request:

    {
      "id": "req-1",
      "type": "request",
      "method": "status",
      "params": {}
    }

Response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "bridge": {
          "version": "0.1.0",
          "state": "running"
        },
        "servers": []
      }
    }

Purpose:

- bridge health;
- later: Mattermost connection health;
- later: WebSocket state;
- later: authenticated user summary without secrets.

## connect

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "connect",
      "params": {
        "server_id": "work",
        "url": "https://mattermost.example.local",
        "login_id": "user@example.local",
        "password": "secret",
        "token": null,
        "ca_file": null
      }
    }

Rules:

- password and token must never be logged;
- server_id is required architecturally;
- MVP may default server_id to "default";
- login/password and token are alternative auth paths;
- saved session token may be loaded by bridge before asking Emacs for credentials.

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "user": {
          "id": "user-id",
          "username": "alice"
        },
        "websocket": "connected"
      }
    }

## me

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "me",
      "params": {
        "server_id": "work"
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "user": {
          "id": "user-id",
          "username": "alice",
          "nickname": "",
          "first_name": "",
          "last_name": ""
        }
      }
    }

## teams

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "teams",
      "params": {
        "server_id": "work"
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "teams": []
      }
    }

## open_self_dm

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "open_self_dm",
      "params": {
        "server_id": "work"
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "channel": {
          "id": "channel-id",
          "type": "direct",
          "display_name": "Self",
          "team_id": null
        }
      }
    }

Implementation note:

Mattermost direct channel creation uses POST /api/v4/channels/direct with two user IDs.

For self-DM MVP, planned body:

    ["my_user_id", "my_user_id"]

This must be verified against Mattermost 10.9.3 during implementation.

## list_channels

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "list_channels",
      "params": {
        "server_id": "work",
        "team_id": "team-id"
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "team_id": "team-id",
        "channels": []
      }
    }

## get_history

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "get_history",
      "params": {
        "server_id": "work",
        "channel_id": "channel-id",
        "limit": 50,
        "before_id": null
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "channel_id": "channel-id",
        "posts": [],
        "order": "ascending",
        "has_more": false
      }
    }

Display rule:

Posts returned to Emacs should be sorted ascending by create_at for easy buffer insertion.

Mattermost REST may return posts in a map plus order array. The bridge normalizes this.

## send_post

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "send_post",
      "params": {
        "server_id": "work",
        "channel_id": "channel-id",
        "message": "hello",
        "root_id": null
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "post": {
          "id": "post-id",
          "channel_id": "channel-id",
          "user_id": "user-id",
          "root_id": null,
          "message": "hello",
          "create_at": 0
        }
      }
    }

## reply_post

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "reply_post",
      "params": {
        "server_id": "work",
        "channel_id": "channel-id",
        "root_id": "root-post-id",
        "message": "reply"
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "post": {
          "id": "reply-post-id",
          "channel_id": "channel-id",
          "user_id": "user-id",
          "root_id": "root-post-id",
          "message": "reply",
          "create_at": 0
        }
      }
    }

## mark_read

Planned request:

    {
      "id": "req-1",
      "type": "request",
      "method": "mark_read",
      "params": {
        "server_id": "work",
        "channel_id": "channel-id"
      }
    }

Planned response:

    {
      "id": "req-1",
      "type": "response",
      "ok": true,
      "result": {
        "server_id": "work",
        "channel_id": "channel-id",
        "read": true
      }
    }

## Planned events

Connection events:

    bridge_started
    server_connected
    server_disconnected
    websocket_connected
    websocket_disconnected
    websocket_reconnecting
    resync_required

Post events:

    post_created
    post_updated
    post_deleted

Channel events:

    channel_updated
    channel_viewed

User/team events:

    user_updated
    team_updated

MVP prioritizes:

    websocket_connected
    websocket_disconnected
    post_created
    post_updated
    post_deleted
    resync_required

## Normalized post shape

    {
      "id": "post-id",
      "server_id": "work",
      "channel_id": "channel-id",
      "user_id": "user-id",
      "root_id": null,
      "message": "raw text",
      "create_at": 0,
      "update_at": 0,
      "delete_at": 0
    }

MVP does not render Markdown.

## Error codes

Planned stable codes:

    invalid_json
    invalid_request
    unknown_method
    unauthorized
    forbidden
    not_found
    rate_limited
    network_error
    tls_error
    server_error
    websocket_disconnected
    internal_error

## Security rules

Never log:

- password;
- session token;
- PAT;
- Authorization header;
- Cookie header;
- raw login request body.

Status responses must not include secrets.

Web status UI must not include secrets.

Token cache files are plain files for MVP, but must be placed under bridge data directory and ignored by git/docker context.
