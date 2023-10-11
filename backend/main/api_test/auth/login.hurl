# not logged in, should deny
GET http://127.0.0.1:3000/
HTTP 401

# login
POST http://127.0.0.1:3000/auth/login
{
    "password": "test1234",
    "id": "test@example.com"
}
HTTP 200
[Asserts]
header "Authorization" not isEmpty
[Captures]
JWT: header "Authorization"

# try again with authorization token
GET http://127.0.0.1:3000/
Authorization: {{JWT}}
HTTP 200hur
[Asserts]
body contains "Hello " # + some uuid