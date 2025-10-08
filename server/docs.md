## DOCS 
HTTP API Documentation for frontend development

### Routes
`GET /status`
`POST /create_user`
`POST /change_password`
`POST /login`
`POST /delete_user`


### Parameters
All data is passed via JSON in the request body.

#### /create_user
- `username`: string, required
- `password`: string, required
- `email`: string, required

#### /change_password
- `email`: string, required
- `old_password`: string, required
- `new_password`: string, required

#### /login
- `email`: string, required
- `password`: string, required

#### /delete_user
- `email`: string, required
- `password`: string, required

### Response
All responses are in JSON format.


```rust
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}
```

the status value can either be "success" or "error". The message field contains additional information about the response.
