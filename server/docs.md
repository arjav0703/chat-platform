## DOCS 
HTTP API Documentation for frontend development

### HTTP Routes
`GET /status`
`POST /create_user`
`POST /change_password`
`POST /login`
`POST /delete_user`

### WebSocket Route
`GET /ws` - WebSocket endpoint for real-time chat messaging


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

## WebSocket API

### Connection
Connect to the WebSocket endpoint at `ws://localhost:8000/ws`

### Message Format

#### Sending Messages (Client -> Server)

**Chat Message:**
```json
{
  "type": "chat",
  "user_email": "user@example.com",
  "content": "Hello, world!"
}
```

**Join Notification:**
```json
{
  "type": "join",
  "user_email": "user@example.com"
}
```

**Leave Notification:**
```json
{
  "type": "leave",
  "user_email": "user@example.com"
}
```

#### Receiving Messages (Server -> Client)

```json
{
  "status": "message",
  "message": {
    "user_email": "user@example.com",
    "username": "John Doe",
    "content": "Hello, world!",
    "timestamp": "2025-10-08T12:34:56.789Z"
  },
  "info": null
}
```

### Features
- Real-time bidirectional communication
- Messages are stored in the database
- Messages are broadcast to all connected clients
- Automatic user lookup from database based on email
- Timestamps in ISO 8601 format
