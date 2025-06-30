A project that showcases **high-throughput file & message transfer over gRPC** together with a React/TypeScript frontend.  
The stack is split into three Rust services plus a React UI:

* **server** – administrative gRPC + REST layer (authentication, user management, routing logic)
* **runner** – bundles two internal services:
  * **client** – exposes an HTTP `/upload` endpoint that fans-out transfers to destination banks over gRPC
  * **destination** – receives inbound transfers, persists files/metadata & acknowledges progress
* **frontend** – simple React dashboard for admins & banks


# Setup

## Generating JWT RSA keys
The auth layer relies on RSA256-signed JWTs. Generate a key-pair once and keep it under the local `keys/` folder (already mapped into the containers):

> keys folder already has a public key, ask for the private key.

```bash
# from the project root
mkdir -p keys
openssl genrsa -out keys/admin_private.pem 4096
openssl rsa -in keys/admin_private.pem -pubout -out keys/admin_public.pem
```

> The paths inside the container are expected to be `/app/keys/admin_private.pem` and `/app/keys/admin_public.pem` – **do not change them unless you also update the corresponding environment variables**.

---

## Environment variables
Each micro-service reads its own `.env` file. **Create the following files next to the listed Cargo.toml before starting the stack.**

> example env files are included.

### 1. `server/.env`
```
# gRPC transport
SERVER_HOST=0.0.0.0
SERVER_PORT=50051

# REST (admin) interface
SERVER_HTTP_HOST=0.0.0.0
SERVER_HTTP_PORT=50052

# MongoDB connection (ENTER admin db uri, this is different from the one in client/destination)
MONGO_URI=

# JWT keys (mounted by compose)
JWT_PRIVATE_KEY_PATH=/app/keys/admin_private.pem
JWT_PUBLIC_KEY_PATH=/app/keys/admin_public.pem
```

### 2. `client/.env`
```
CLIENT_HOST=0.0.0.0
# use the same port for "VITE_CLIENT_API_URL"
CLIENT_PORT=8081

# Points to the *server* gRPC endpoint *inside* the docker compose network
SERVER_HOST=ftp-grpc-server
SERVER_PORT=50051

# this URI is the same as destination
MONGO_URI=

# JWT
JWT_PUBLIC_KEY_PATH=/app/keys/admin_public.pem
```

### 3. `destination/.env`
```
DESTINATION_HOST=0.0.0.0
DESTINATION_PORT=50053

# this URI is the same as client
MONGO_URI=
```

### 4. `frontend/.env`
```
# React-Vite variables (see frontend/src/lib/api.ts)
VITE_CLIENT_API_URL=http://localhost:8081
VITE_SERVER_API_URL=http://ftp-grpc-server:50052
```

> MONGO_URI for client/destination is the same, but server will have a different MONGO_URI.

---

## Running with Docker-Compose

### Quick start (pre-built images)
If you already have the `ftp-grpc-<service>` images published locally or pulled from a registry:

```bash
docker compose up -d            # uses docker-compose.yml
```

### Build images locally (recommended for development)

```bash
# Build the images and start the whole stack using the build-oriented compose file

docker compose -f docker-compose.build.yml build

docker compose -f docker-compose.build.yml up -d
```

---

# API Reference

## 1. Admin Server (actix-web)

| Method | Path | Auth | Purpose | Required headers / fields |
|--------|------|------|---------|---------------------------|
| POST | `/login` | _No_ | Obtain JWT. | `username`, `password` headers |
| POST | `/api/add` | Admin | Add a new user (Bank or Admin). | Headers: `username`, `password`, `ip` (IP mandatory for Bank users) |
| POST | `/api/update` | Admin | Update user password and/or IP. | Headers: `username` and optional `password`, `ip` |
| POST | `/api/delete` | Admin | Remove a user. | Header: `username` |
| GET  | `/api/available` | Any JWT | List all available bank destinations. | – |
| GET  | `/api/users` | Admin | Full list of banks & admins. | – |
| GET  | `/api/file-info` | Any JWT | Fetch stored file metadata (all banks). | – |
| POST | `/api/admin-upload` | Admin | Send file or text message to selected banks. | **multipart/form-data** fields:<br>• `file` – binary file (optional)<br>• `message` – text (optional)<br>• `destinations` – JSON string `[ {"username":"BANK_D", "ip":"127.0.0.1"}, ... ]`<br>• `sender` – your admin username |

## 2. Bank Server (actix-web)

| Method | Path | Auth | Purpose | Required headers / fields |
|--------|------|------|---------|---------------------------|
| POST | `/upload` | Bank JWT | Upload a file and/or message to other banks via the Admin server. | **multipart/form-data** fields identical to `/api/admin-upload` |
| GET  | `/file-info` | Bank JWT | Retrieve this bank's transfer history. | – |

## 3. gRPC Transfer Service
* **Proto file**: `proto/ftp.proto`
* Services are exposed on:
  * Admin Server → `grpc://localhost:50051`
  * Destination Bank → `grpc://localhost:50053`

```
service TransferService {
  rpc Transfer(stream TransferRequest) returns (stream TransferResponse);
}
```
Use any gRPC client (e.g. `grpcurl`, Postman) to interact. Maximum message size is **8 mb**.

## Architecture Overview

1. **Bank ➜ Bank transfer (with Server in the middle)**
  - A Bank runs the **runner** container which bundles two micro-services:
    - **client** – exposes a simple HTTP `POST /upload` endpoint for the local Bank UI or CLI.
    - **destination** – receives inbound gRPC streams from other peers.
  - When a Bank user uploads a file/message, *client* breaks the payload into chunks and opens a bidirectional gRPC stream (`TransferService.Transfer`) to the **server**.
  - The server acts as a smart router: it checks the JWT, looks up the requested recipient Banks in its MongoDB, then **fans-out** the same stream to each recipient's *destination* service.
  - Each destination writes the incoming chunks to disk / DB, sends progress back through the stream, and finally ACKs success or failure.

2. **Admin ➜ Bank transfer**
  - Admins talk to the **server** directly over HTTP (`/api/admin-upload`).
  - The server reuses the exact same gRPC fan-out pipeline described above to push the payload to one or many Banks.

3. **Transport & APIs**
  - gRPC (powered by `tonic`) is used for the heavy-lifting: it enables 
     bidirectional streaming and 8 MB message sizes.
  - actix-web powers all REST endpoints (`/login`, `/upload`, etc.).
  - JWT (RSA-256) secures every hop – Banks and Admins present the token on both HTTP and gRPC calls.

4. **Why this matters**
  - **Multi-Bank:** one push can target any subset of Banks; adding a new Bank is just a DB insert.
  - **Resilience:** chunked streaming means large files resume on network hiccups (server can ask for `RETRY`).
  - **Observability:** every transfer is timestamped and stored, so transfer history is queryable via REST.