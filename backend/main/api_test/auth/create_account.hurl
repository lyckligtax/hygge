# create a user
POST http://127.0.0.1:3000/auth/create_account
{
    "password": "test1234",
    "id": "test@example.com"
}
HTTP 201
