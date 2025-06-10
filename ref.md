i'm working on a project that involves transferring files from bank A to bank B. the tech stack i'm using includes a rust backend & gRPC.

---

the gist of the project:

let's say we have A, and A wants to send a file to C. i'm building a system that allows A to send files to C, but with B intercepting this request. essentially, while the file is on its way from A to C, it passes through B. B isn't a user — you can think of it as an administrative server. its main role is to intercept and forward files to wherever they're meant to go. it always sits in the middle of the transfer.


> NOTE: we are allowing file transfer, messages or BOTH at the same time.

---

Internals of the File Transfer System

1. Initial Request & UI Interaction

* The user (say, User A) accesses the application via a web-based **UI** served by an **Actix Web** server.
* The user fills out a form and uploads the file intended for transfer.
* Once submitted, the file upload request is received by the **Actix Web server**, which is running on **A’s own machine/server**.

---

2. Temporary Storage on Sender's Server (A)

* Upon receiving the file, the Actix handler stores the uploaded file temporarily **on disk**.
* The exact location/path of this temporary storage is yet to be finalized, but it will reside on **User A’s server**.

---

3. Initiating File Transfer to B via gRPC

* After storing the file, the Actix server triggers a **gRPC client**, which is responsible for sending the file to the next hop, i.e., **User B’s server**.
* The file is not sent all at once — it is transferred **in chunks**.

  * The **chunk size** is configurable and can be decided later.

---

4. Chunked Transfer Logic (A → B)

* The gRPC client at A sends the **first chunk** of the file to B.
* B receives the chunk and writes it to temporary storage **on B's server**.
* Once B has successfully written the chunk, it sends an **acknowledgment** back to A.
* Upon receiving the acknowledgment, A sends the **next chunk**.
* This process continues until the entire file is transferred.

---

5. Forwarding from B to C (Same Chunked Logic)

* Once the full file has been received and written by B:

  * The gRPC component running on **B’s server** automatically starts the next transfer to **User C’s server**.
  * The file transfer from B to C follows **the same chunked pattern** as A to B, including:

    * Chunk-wise write
    * Acknowledgments after each chunk
    * Temporary storage on **C's server** as well.

---

6. Code Deployment & Consistency

* The **entire application (UI, Actix server, gRPC client & server, chunk handling logic)** is present and deployed **on all three users’ servers** (A, B, and C).
* Temporary files (during both upload and transfer) are always stored locally **on the respective user's server**.

---

According to current architecture, we have three folders, `client`, `server` and `destination`. The client/destination folders will be present on the user's server. (i.e. the client will have both folders and the destination will have both the folders). The user will then run the client and destination code on different ports.

> NOTE: we can run the client/destination on different channels (i.e. mpsc channels) and assign them different ports.

There will only be ONE and only one server, the server code won't be elsewhere.