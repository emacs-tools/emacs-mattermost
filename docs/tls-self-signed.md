# TLS and Self-Signed Certificates

## Context

Corporate Mattermost may use a certificate signed by an internal CA.

The user currently has Emacs-side workaround settings:

    (setq gnutls-verify-error nil)
    (setq tls-checksig-warn-clock-skew t)
    (setq gnutls-trustfiles '("/etc/ssl/certs/ca-certificates.crt"))

These settings affect Emacs network connections.

They do not affect the Rust bridge running in Docker.

## Bridge policy

The Rust bridge must not disable TLS verification by default.

Preferred approach:

- trust system CA store;
- allow explicit additional CA file;
- document Docker mount;
- return clear tls_error on failure.

## Planned configuration

Environment variable:

    MM_CA_FILE=/certs/company-ca.crt

Compose volume:

    ./certs/company-ca.crt:/certs/company-ca.crt:ro

The CA file should be a PEM certificate.

## Emacs connection to bridge

Emacs connects to the local bridge over plain TCP:

    127.0.0.1:8765

This local connection does not use TLS in MVP.

TLS is needed for bridge → Mattermost HTTPS/WebSocket traffic.

## WebSocket TLS

Mattermost WebSocket URL is derived from server URL:

    https://mattermost.example.local -> wss://mattermost.example.local/api/v4/websocket
    http://mattermost.example.local  -> ws://mattermost.example.local/api/v4/websocket

The same CA policy must apply to REST and WebSocket connections.

## Clock skew

Certificate validation can fail if system clock is wrong.

The bridge should report this clearly when possible.

Emacs setting:

    tls-checksig-warn-clock-skew

does not fix Rust bridge certificate validation.

## Debugging checklist

If bridge cannot connect to Mattermost over HTTPS:

1. Check server URL.
2. Check container can resolve hostname.
3. Check container can reach server port.
4. Check CA file is mounted.
5. Check MM_CA_FILE points to the mounted path.
6. Check certificate chain.
7. Check container clock.
8. Check Mattermost reverse proxy configuration.

## Non-goals for MVP

- automatic certificate discovery from Emacs settings;
- disabling verification via simple flag;
- certificate pinning;
- mTLS client certificates.

mTLS can be added later if corporate infrastructure requires it.
