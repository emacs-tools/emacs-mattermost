# Docker Runtime

## Decision

Docker Compose is the primary way to run the Rust bridge.

Emacs does not start containers.

The user starts the bridge separately, then Emacs connects to it over localhost TCP.

## Planned ports

Bridge JSONL TCP port:

    8765

Web status HTTP port:

    8766

Planned local bindings:

    127.0.0.1:8765:8765
    127.0.0.1:8766:8766

The bridge should not be exposed publicly by default.

## Planned compose usage

Start:

    docker compose up -d --build

Stop:

    docker compose down

Logs:

    docker compose logs -f mm-bridge

Status page:

    http://127.0.0.1:8766/

Health endpoint:

    http://127.0.0.1:8766/health

Status JSON:

    http://127.0.0.1:8766/status

## Planned volumes

Persistent bridge data:

    ./data:/data

Purpose:

- session token cache;
- PAT cache;
- lightweight metadata;
- future cache files.

MVP token files are plain files without encryption.

They must not be committed to git.

## Planned environment variables

Bridge listen address:

    MM_BRIDGE_LISTEN=0.0.0.0:8765

Web listen address:

    MM_WEB_LISTEN=0.0.0.0:8766

Data directory:

    MM_DATA_DIR=/data

Default Mattermost URL:

    MM_SERVER_URL=https://mattermost.example.local

Default server id:

    MM_SERVER_ID=work

Optional CA file inside container:

    MM_CA_FILE=/certs/company-ca.crt

Optional PAT file path:

    MM_TOKEN_FILE=/data/tokens/work.token

Do not put passwords directly into compose.yaml.

Do not print environment variables wholesale in logs.

## Self-signed / internal CA

Mount CA certificate read-only:

    ./certs/company-ca.crt:/certs/company-ca.crt:ro

Then set:

    MM_CA_FILE=/certs/company-ca.crt

The Rust bridge must validate certificates by default.

Disabling verification globally is not the default design.

Emacs TLS settings do not affect bridge TLS validation.

## Secrets

MVP allows plain token cache files under ./data.

Still required:

- ignore ./data in git;
- ignore secrets in docker build context;
- never log tokens;
- never expose tokens in web status;
- avoid putting passwords in compose.yaml.

Password may come from Emacs ~/.authinfo and be sent to bridge in connect request.

The bridge must not log the connect request body.

## Future scripts

Planned scripts:

    scripts/docker-build.sh
    scripts/docker-up.sh
    scripts/docker-down.sh

They will be implemented after the minimal bridge compiles and responds to ping/status.

## Current status

Docker files are placeholders until the Rust bridge skeleton is implemented.
