i'm working on a project which involves file transferring from bank A to bank B.
the tech stack involves rust backend, gRPC. 

gist of the project: 

say we have A, A wants to send a file to C. I am building a system which allows A
to send files to C, but B interccepts this request. Essentially A, while sending 
the file to C, the file is met by B.

Internals:

The user is welcomed by the UI. user fills the form or wtv, then uploads the file 
for transfer. then, the server code (this code is running on the server of
the user itself, that's right, according to our earlier analogy, A will run the code
on their server) will get the request. This request is handled by the actix web server,
which then invokes the gRPC client. But before invoking the gRPC client to start 
the server, the file, which is to be transferred, is stored temporarily somewhere on
disk (on the server, the whereabouts of this are not known yet).

Once the file is stored temporarily, we start the trasnfer through the gRPC client.

IMPORTANT part, the transfer of the files is done in Chunks (size of the chunks will 
be decided later ig). the first chunk will be sent, then this chunk is written in the 
server of B. once the write is done completely, B will send an acknowledgement to A
then A's gRPC sends the next chunk. and so on. now the file is temporarily stored
B.

Once the write is completely done in B, since B already knows the destination, the 
gRPC in B will start the transfer to C. transfer is done in the same way as A to B.
just like A was running the code, B also will run the code on their server and so will C.


---

- somehow indicate that the msg/file has ended
    - one way to do this is use EOF indicator
