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

**⚠️ IMPORTANT: Authentication Required**
The first message sent after connecting MUST be an authentication message. The connection will be closed if authentication fails or if any other message type is sent first.

### Message Format

#### Sending Messages (Client -> Server)

**Authentication Message (MUST BE FIRST):**
```json
{
  "type": "auth",
  "email": "user@example.com",
  "password": "your_password"
}
```

**Chat Message:**
```json
{
  "type": "chat",
  "content": "Hello, world!"
}
```

**Join Notification:**
```json
{
  "type": "join"
}
```

**Leave Notification:**
```json
{
  "type": "leave"
}
```

#### Receiving Messages (Server -> Client)

**Authentication Success:**
```json
{
  "status": "authenticated",
  "message": null,
  "info": "Welcome, John Doe!"
}
```

**Authentication Failed:**
```json
{
  "status": "error",
  "message": null,
  "info": "Authentication failed: Invalid email or password"
}
```

**Chat Message:**
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
- **Secure authentication required** - Users must authenticate with email and password
- Real-time bidirectional communication
- Messages are stored in the database
- Messages are broadcast to all connected clients
- User information is validated against the database
- Timestamps in ISO 8601 format

### Authentication Flow
1. Client connects to WebSocket endpoint
2. Client sends authentication message with email and password
3. Server validates credentials against database
4. If valid: Server sends success response, connection stays open
5. If invalid: Server sends error response, connection closes
6. After authentication, client can send chat messages

