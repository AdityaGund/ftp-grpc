# ftp-grpc

## test

- curl commands to test endpoint `/upload`
- make sure to create a file inside `client/`, call it `client/send_files`
- add a test text file called `my-test.txt`
- run the below command

```bash
curl -F "file=@client/send_files/my-test.txt" -F "message=Hello world" -F "destination=BANK_C" http://127.0.0.1:8080/upload
```

> to test only file/msg, just remove the parameter from the curl command.