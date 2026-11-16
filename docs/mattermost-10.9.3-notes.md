# Mattermost 10.9.3 Notes

## Target server

Target Mattermost server version: 10.9.3.

All MVP implementation must be verified against this version.

## REST base path

    /api/v4

## Authentication

Login endpoint:

    POST /api/v4/users/login

Request body:

    {
      "login_id": "user@example.local",
      "password": "secret"
    }

Successful login returns:

- user object in response body;
- session token in Token response header.

Future requests use:

    Authorization: Bearer <token>

PAT uses the same bearer header.

MVP supports:

- login/password;
- saved session token;
- PAT.

MFA is not part of MVP unless required by the target server.

## Current user

    GET /api/v4/users/me

The string me can be used instead of current user id in relevant endpoints.

## Teams

Planned endpoint:

    GET /api/v4/users/me/teams

Used by bridge method:

    teams

## Channels

Planned endpoints:

    GET /api/v4/users/{user_id}/teams/{team_id}/channels
    GET /api/v4/channels/{channel_id}
    GET /api/v4/channels/{channel_id}/posts

Direct channel creation:

    POST /api/v4/channels/direct

Request body:

    ["user_id_1", "user_id_2"]

For self-DM MVP:

    ["my_user_id", "my_user_id"]

This must be tested on Mattermost 10.9.3.

If self-DM creation is rejected by the server, fallback behavior must be decided explicitly before changing MVP scope.

## Posts

Create post:

    POST /api/v4/posts

Request body:

    {
      "channel_id": "channel-id",
      "message": "raw text",
      "root_id": ""
    }

Reply in thread:

    {
      "channel_id": "channel-id",
      "message": "reply text",
      "root_id": "root-post-id"
    }

MVP sends raw text.

No Markdown rendering in Emacs MVP.

## History normalization

Mattermost channel posts response commonly contains:

- order: array of post ids;
- posts: map from post id to post object.

Bridge must normalize this into an ordered posts array.

Display order for Emacs:

    ascending by create_at

Pagination:

    before_id
    limit

MVP can start with latest N posts.

## Mark read

Planned endpoint:

    POST /api/v4/channels/view

Request body:

    {
      "channel_id": "channel-id"
    }

Exact response shape must be verified during implementation.

Bridge method:

    mark_read

## WebSocket

Endpoint:

    /api/v4/websocket

Authentication challenge:

    {
      "seq": 1,
      "action": "authentication_challenge",
      "data": {
        "token": "mattermosttokengoeshere"
      }
    }

Successful response:

    {
      "status": "OK",
      "seq_reply": 1
    }

After authentication, server sends hello event.

Event envelope:

    {
      "event": "posted",
      "data": {},
      "broadcast": {},
      "seq": 0
    }

## WebSocket events for MVP

Important:

    hello
    posted
    post_edited
    post_deleted
    channel_viewed
    channel_updated

Ignored for MVP:

    typing
    reaction_added
    reaction_removed

The bridge may log ignored event names at debug level, but must not fail on unknown events.

## posted event

The posted event usually contains a JSON-encoded post string inside data.post.

Bridge must parse it defensively:

- data.post may be a string containing JSON;
- parse failure should produce a structured internal warning;
- do not crash the WebSocket loop.

## Reconnect

WebSocket reconnect is owned by bridge.

On reconnect:

1. reopen WebSocket;
2. authenticate;
3. wait for hello or OK;
4. mark connection healthy;
5. emit websocket_connected;
6. trigger resync for open buffers if needed.

Mattermost server remains source of truth.

Do not rely only on missed WebSocket events for correctness.

## Forced refresh

Emacs g triggers bridge forced refresh.

Bridge should:

- check REST availability;
- check WebSocket health;
- reconnect if needed;
- refetch requested buffer data;
- return fresh server state.

## Rate limits

Mattermost may return:

    429 Too Many Requests

Headers may include:

    X-Ratelimit-Limit
    X-Ratelimit-Remaining
    X-Ratelimit-Reset

Bridge should map this to:

    rate_limited

MVP can surface the error without sophisticated retry scheduling.

## Error shape

Mattermost API errors commonly look like:

    {
      "id": "the.error.id",
      "message": "Something went wrong",
      "request_id": "",
      "status_code": 400
    }

Bridge must normalize errors and never leak auth headers.

## Verification tasks

During implementation, verify on target server:

- login/password works;
- Token header is returned;
- saved session token works after restart;
- PAT works if enabled;
- self-DM channel can be created/opened;
- posting to self-DM works;
- posted event arrives for own messages;
- mark_read endpoint works;
- internal CA configuration works from Docker.
