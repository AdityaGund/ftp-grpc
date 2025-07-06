A project that showcases **high-throughput file & message transfer over gRPC** together with a React/TypeScript frontend.  
The stack is split into three Rust services plus a React UI:

* **server** – administrative gRPC + REST layer (authentication, user management, routing logic)
* **runner** – bundles two internal services:
  * **client** – exposes an HTTP `/upload` endpoint that fans-out transfers to destination banks over gRPC
  * **destination** – receives inbound transfers, persists files/metadata & acknowledges progress
* **frontend** – simple React dashboard for admins & banks


# Setup

## Prerequisites

* Docker & Docker Compose v2
* OpenSSL (only required once to generate the JWT key-pair)
* Optional – Rust toolchain & Node.js if you prefer running services outside Docker

## 1. Clone the repository

```bash
git clone https://github.com/AdityaGund/ftp-grpc.git
cd ftp-grpc
```

## 2. Generate JWT RSA keys

Create an RSA-256 key-pair **once** under the `keys/` folder (already mounted into every container):

> Only share **public** key with banks.

```bash
mkdir -p keys
openssl genrsa -out keys/admin_private.pem 4096
openssl rsa -in keys/admin_private.pem -pubout -out keys/admin_public.pem
```

The paths inside the containers are fixed to `/app/keys/admin_private.pem` and `/app/keys/admin_public.pem` – change them only if you also update the corresponding `JWT_*_KEY_PATH` variables.

## 3. Configure environment variables

Each micro-service contains a ready-made `.env.example`. Copy it to `.env` and tweak as needed:

```bash
cp server/.env.example       server/.env
cp client/.env.example       client/.env
cp destination/.env.example  destination/.env
cp runner/.env.example  runner/.env
cp frontend/.env.example     frontend/.env
```

### **Databases:** you will need **two MongoDB databases** for local/testing purposes:

1. **Admin DB** – used exclusively by the `server` service (admin & routing data).
2. **Bank DB**  – shared by both the `client` and `destination` services (transfer metadata).

They can live on the same Mongo instance – simply point each `.env` file to the appropriate database name.

## 4. Start the stack

### Pre-built images
If you already have the `ftp-grpc-*` images locally (or pulled from a registry):

```bash
docker compose up -d
```

### Build everything locally (recommended for development)

```bash
docker compose -f docker-compose.build.yml build

docker compose -f docker-compose.build.yml up -d
```

Once the containers are up you can reach:

* Admin REST API → `http://localhost:50052`
* Bank client API → `http://localhost:8081`
* React UI        → `http://localhost:5173` (if you mapped the port)

## 5. Seed the Admin database (mandatory)

Before logging in to the dashboard you **must** create at least one Admin account in the `admin_db.admin` collection. Without it the authentication endpoint will return an error.

1. Ensure the stack is running:

   ```bash
   docker compose up -d   # or the build variant if you built the images locally
   ```

2. Open an interactive Mongo shell inside the `mongoAdmin` container (password: `adminPass123`):

   ```bash
   docker exec -it mongoAdmin mongosh -u mongoAdmin -p adminPass123 --authenticationDatabase admin
   ```

3. Insert an admin document (adjust the ObjectId / credentials as you like):

   ```js
   use admin_db;
   db.admin.insertOne({
     _id: ObjectId("685258ebc25f4303af50ddf2"),
     username: "A001",
     password: "$argon2id$v=19$m=19456,t=2,p=1$GTwEdGQ07tZ1zOWLU8UShQ$5M3mYiVPgnR7nsH3rm7Orcdj24V8xGL+AZIHv1Uafwo"
   });
   ```

   > the above password is `testpass123` hashed using argon2 crate. change it later using UI.

   You can also run the helper script `playground-1.mongodb.js` (VS Code MongoDB playground) or any MongoDB client using the connection URI:

   ```
   mongodb://mongoAdmin:adminPass123@localhost:27017/admin_db?authSource=admin
   ```

Once at least one Admin exists you can log in (`POST /login`) and use the UI.

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

### Quick testing inside Docker

* **Frontend → Server**: make sure `VITE_SERVER_API_URL` points to
  `ftp-grpc-server:50052` (already shown above). This is the service name
  reachable from the browser running inside the *frontend* container.

* **Ad-hoc file transfers from your host → runner**: send gRPC/HTTP requests to
  an address that's **inside the compose network** – either use the service name
  `ftp-grpc-runner` or the container's IP (obtain with
  `docker inspect ftp-grpc-runner`). Requests to `localhost` won't work because
  they bypass the Docker bridge.