# WebSocket Authentication Testing Guide

## Setup

1. **Start the server:**
   ```bash
   cd /Users/arjav/Documents/chat-app/server
   cargo run
   ```

2. **Create a test user** (if you haven't already):
   ```bash
   curl -X POST http://localhost:8000/create_user \
     -H "Content-Type: application/json" \
     -d '{
       "username": "Test User",
       "email": "test@example.com",
       "password": "password123"
     }'
   ```

## Testing with the HTML Client

1. Open `test_websocket.html` in your browser (you can open multiple tabs to simulate multiple users)
2. Enter your email and password
3. Click "Connect"
4. If authentication succeeds, you'll see "Welcome, [Your Name]!" message
5. Start chatting!

## Authentication Flow

### Step 1: Connect to WebSocket
```javascript
const ws = new WebSocket('ws://localhost:8000/ws');
```

### Step 2: Send Authentication Message (MUST BE FIRST)
```javascript
ws.onopen = function() {
    const authMsg = {
        type: 'auth',
        email: 'test@example.com',
        password: 'password123'
    };
    ws.send(JSON.stringify(authMsg));
};
```

### Step 3: Handle Authentication Response
```javascript
ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    
    if (data.status === 'authenticated') {
        console.log('Success!', data.info);
        // Now you can send chat messages
    } else if (data.status === 'error') {
        console.error('Auth failed:', data.info);
        // Connection will be closed by server
    } else if (data.status === 'message') {
        // Handle incoming chat messages
        console.log('Message:', data.message);
    }
};
```

### Step 4: Send Chat Messages (Only After Authentication)
```javascript
const chatMsg = {
    type: 'chat',
    content: 'Hello, world!'
};
ws.send(JSON.stringify(chatMsg));
```

## Security Features

✅ **Password validation** - Passwords are hashed with SHA256 before comparison
✅ **Database validation** - User credentials are verified against the database
✅ **Connection enforcement** - Unauthenticated users cannot send messages
✅ **Automatic disconnection** - Failed authentication closes the connection
✅ **Session management** - Authenticated user info is stored for the connection lifetime

## Error Scenarios

1. **Wrong password/email:**
   - Server responds with error status
   - Connection is closed automatically

2. **Missing authentication:**
   - If you send any message other than auth first
   - Server responds with error and closes connection

3. **Non-existent user:**
   - Treated same as wrong password
   - Connection is closed

## Message Types After Authentication

### Chat Message
```json
{
  "type": "chat",
  "content": "Your message here"
}
```

### Join Notification
```json
{
  "type": "join"
}
```

### Leave Notification
```json
{
  "type": "leave"
}
```

## Testing Multiple Users

1. Open multiple browser tabs or windows
2. Use different user credentials in each
3. Send messages from one tab
4. See them appear in all other authenticated tabs in real-time!
