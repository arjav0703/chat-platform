import { useState, useEffect, useRef } from "react";
import "./App.css";
import { Client, Stronghold } from '@tauri-apps/plugin-stronghold';
import { appDataDir } from '@tauri-apps/api/path';

interface ChatMessage {
  user_email: string;
  username: string;
  content: string;
  timestamp: string;
}

interface WsResponse {
  status: string;
  message?: ChatMessage;
  info?: string;
}

interface SystemMessage {
  type: 'system';
  content: string;
  timestamp: Date;
}

type Message = ChatMessage | SystemMessage;

function App() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [messageContent, setMessageContent] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [status, setStatus] = useState<'disconnected' | 'connecting' | 'connected'>('disconnected');
  const [statusText, setStatusText] = useState('Disconnected');

  const wsRef = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<HTMLDivElement | null>(null);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Cleanup WebSocket on unmount
  useEffect(() => {
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  const addSystemMessage = (content: string) => {
    setMessages(prev => [...prev, {
      type: 'system',
      content,
      timestamp: new Date()
    }]);
  };

  const addChatMessage = (message: ChatMessage) => {
    setMessages(prev => [...prev, message]);
  };

  const updateStatus = (text: string, state: 'disconnected' | 'connecting' | 'connected') => {
    setStatusText(text);
    setStatus(state);
  };

  const fetchPreviousMessages = async (limit: number = 100) => {
    try {
      addSystemMessage('Loading previous messages...');
      const response = await fetch(`http://localhost:8000/messages?limit=${limit}`);
      const data = await response.json();

      if (data.status === 'success' && data.messages) {
        // Add all previous messages
        const previousMessages: ChatMessage[] = data.messages.map((msg: any) => ({
          user_email: msg.user_email,
          username: msg.username,
          content: msg.content,
          timestamp: msg.timestamp
        }));

        setMessages(prev => [...prev, ...previousMessages]);
        addSystemMessage(`Loaded ${previousMessages.length} previous message(s)`);
      } else {
        addSystemMessage('No previous messages found');
      }
    } catch (error) {
      console.error('Error fetching messages:', error);
      addSystemMessage('Failed to load previous messages');
    }
  };

  const connect = () => {
    if (!email || !password) {
      alert('Please enter your email and password');
      return;
    }

    updateStatus('Connecting...', 'connecting');
    const ws = new WebSocket('ws://localhost:8000/ws');
    wsRef.current = ws;

    ws.onopen = () => {
      updateStatus('Authenticating...', 'connecting');
      addSystemMessage('Connected to chat server, authenticating...');

      // send authentication message first
      const authMsg = {
        type: 'auth',
        email: email,
        password: password
      };
      ws.send(JSON.stringify(authMsg));
    };

    ws.onmessage = (event) => {
      const data: WsResponse = JSON.parse(event.data);

      if (data.status === 'authenticated') {
        updateStatus('Connected & Authenticated', 'connected');
        addSystemMessage(data.info || 'Authentication successful!');

        // Fetch previous messages
        fetchPreviousMessages(100);

        // send join notification after authentication
        const joinMsg = {
          type: 'join'
        };
        ws.send(JSON.stringify(joinMsg));
      } else if (data.status === 'error') {
        updateStatus('Error: ' + (data.info || 'Unknown error'), 'disconnected');
        addSystemMessage('Error: ' + (data.info || 'Unknown error'));
        ws.close();
      } else if (data.status === 'message' && data.message) {
        addChatMessage(data.message);
      }
    };

    ws.onclose = () => {
      updateStatus('Disconnected', 'disconnected');
      addSystemMessage('Disconnected from chat server');
      wsRef.current = null;
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      addSystemMessage('Error connecting to server');
    };
  };

  const disconnect = () => {
    if (wsRef.current) {
      // Send leave notification
      const leaveMsg = {
        type: 'leave'
      };
      wsRef.current.send(JSON.stringify(leaveMsg));
      wsRef.current.close();
      wsRef.current = null;
    }
  };

  const sendMessage = () => {
    if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
      alert('Not connected to server');
      return;
    }

    if (!messageContent.trim()) {
      alert('Please enter a message');
      return;
    }

    const message = {
      type: 'chat',
      content: messageContent
    };

    wsRef.current.send(JSON.stringify(message));
    setMessageContent('');
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      sendMessage();
    }
  };

  const isSystemMessage = (msg: Message): msg is SystemMessage => {
    return 'type' in msg && msg.type === 'system';
  };

  useEffect(() => {
    async function setup_stronghold() {
      const { stronghold, client } = await initStronghold();
      console.log(stronghold, client);

      insertRecord(client, 'my_key', 'my_value');
      const value = await getRecord(client, 'my_key');
      console.log('Retrieved value from Stronghold:', value);

      insertRecord(client, 'email', email);
      insertRecord(client, 'password', password);

    }

    setup_stronghold();
  }, []);


  return (
    <div className="chat-container">
      <h1>Chat</h1>
      <div className={`status status-${status}`}>
        {statusText}
      </div>

      <div className="auth-controls">
        <input
          type="text"
          placeholder="Your email (e.g., user@example.com)"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          disabled={status === 'connected'}
        />
        <input
          type="password"
          placeholder="Your password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          disabled={status === 'connected'}
        />
        <button
          onClick={connect}
          disabled={status !== 'disconnected'}
        >
          Connect
        </button>
        <button
          onClick={disconnect}
          disabled={status === 'disconnected'}
        >
          Disconnect
        </button>
        <button
          onClick={() => fetchPreviousMessages(100)}
          disabled={status === 'disconnected'}
          title="Load previous messages"
        >
          Load History
        </button>
      </div>

      <div className="messages-container">
        {messages.map((msg, index) => {
          if (isSystemMessage(msg)) {
            return (
              <div key={index} className="message system-message">
                {msg.content}
              </div>
            );
          } else {
            const timestamp = new Date(msg.timestamp).toLocaleString();
            return (
              <div key={index} className="message chat-message">
                <div className="message-header">
                  {msg.username} ({msg.user_email})
                </div>
                <div className="message-content">{msg.content}</div>
                <div className="message-time">{timestamp}</div>
              </div>
            );
          }
        })}
        <div ref={messagesEndRef} />
      </div>

      <div className="message-input">
        <input
          type="text"
          placeholder="Type your message..."
          value={messageContent}
          onChange={(e) => setMessageContent(e.target.value)}
          onKeyPress={handleKeyPress}
          disabled={status !== 'connected'}
        />
        <button
          onClick={sendMessage}
          disabled={status !== 'connected'}
        >
          Send Message
        </button>
      </div>
    </div>
  );
}

export default App;

const initStronghold = async () => {
  const vaultPath = `${await appDataDir()}/vault.hold`;
  const vaultPassword = 'vault password';
  const stronghold = await Stronghold.load(vaultPath, vaultPassword);

  let client: Client;
  const clientName = 'name your client';
  try {
    client = await stronghold.loadClient(clientName);
  } catch {
    client = await stronghold.createClient(clientName);
  }

  return {
    stronghold,
    client,
  };
};

async function insertRecord(store: any, key: string, value: string) {
  const data = Array.from(new TextEncoder().encode(value));
  await store.insert(key, data);
}

// Read a record from store
async function getRecord(store: any, key: string): Promise<string> {
  const data = await store.get(key);
  return new TextDecoder().decode(new Uint8Array(data));
}



