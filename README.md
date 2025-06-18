# ftp-grpc

## test

- curl commands to test endpoint `/upload`
- make sure to create a file inside `client/`, call it `client/send_files`
- add a test text file called `my-test.txt`
- run the below command

```bash
curl -F "file=@C:/Users/paras/Desktop/rust/projects/ftp-grpc/client/send_files/my-test.txt" -F "message=Hello world" -F "destination=BANK_D" http://127.0.0.1:8080/upload
```

```bash
curl -F "file=@C:/Users/paras/Downloads/4.4.0.apk" -F "message=Hello world" -F "destination=BANK_D" http://127.0.0.1:8080/upload
```

> to test only file/msg, just remove the parameter from the curl command.

### add user (Bank or Admin User)

```bash
curl -v -X POST \
  -H "username: testuser" \
  -H "password: testpass123" \
  -H "role: Admin" \
  http://127.0.0.1:50052/login
```

### login

```bash
# add an **admin** user
TOKEN=$(curl -s -X POST http://127.0.0.1:50052/login -H "username: admin1" -H "password: secret" -H "role: admin" | jq -r .token)
```