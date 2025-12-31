import React, { useState, useRef, useEffect } from 'react';
import { MessageCircle, Send } from 'lucide-react';
import styles from './ChatPanel.module.css';

interface Message {
  username: string;
  text: string;
  timestamp: string;
}

interface ChatPanelProps {
  username: string;
}

export const ChatPanel: React.FC<ChatPanelProps> = ({ username }) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputText, setInputText] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    // Use the API URL from environment for both dev and prod
    const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080';
    const apiHost = new URL(apiUrl).hostname;
    const apiProtocol = new URL(apiUrl).protocol === 'https:' ? 'wss:' : 'ws:';
    const apiPort = new URL(apiUrl).port;
    
    // Build WebSocket URL using API domain
    const wsUrl = apiPort 
      ? `${apiProtocol}//${apiHost}:${apiPort}/api/chat/ws`
      : `${apiProtocol}//${apiHost}/api/chat/ws`;

    console.log('Connecting to WebSocket:', wsUrl);

    // Connect to WebSocket
    const ws = new WebSocket(wsUrl);
    let pingInterval: number | undefined;

    ws.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);

      // Send ping every 30 seconds to keep connection alive
      pingInterval = window.setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          console.log('Sending keepalive ping');
          ws.send(JSON.stringify({ type: 'ping' }));
        }
      }, 30000);
    };

    ws.onmessage = (event) => {
      try {
        const message: Message = JSON.parse(event.data);
        setMessages((prev) => [...prev, message]);
      } catch (error) {
        console.error('Failed to parse message:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      console.error('Failed to connect to:', wsUrl);
    };

    ws.onclose = (event) => {
      console.log('WebSocket disconnected', { code: event.code, reason: event.reason || 'No reason provided' });
      setIsConnected(false);
      if (pingInterval) {
        clearInterval(pingInterval);
      }
    };

    wsRef.current = ws;

    // Cleanup on unmount
    return () => {
      if (pingInterval) {
        clearInterval(pingInterval);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  const handleSendMessage = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (inputText.trim() && wsRef.current && isConnected) {
      const message: Message = {
        username,
        text: inputText.trim(),
        timestamp: new Date().toISOString(),
      };

      // Send to WebSocket server
      wsRef.current.send(JSON.stringify(message));
      setInputText('');
    }
  };

  const formatTime = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString('en-US', { 
        hour: '2-digit', 
        minute: '2-digit' 
      });
    } catch {
      return '';
    }
  };

  return (
    <aside className={styles.chatPanel}>
      <div className={styles.chatHeader}>
        <MessageCircle size={18} className={styles.chatIcon} />
        <h3 className={styles.chatTitle}>Global Chat</h3>
        <span className={`${styles.connectionStatus} ${isConnected ? styles.connected : styles.disconnected}`}>
          {isConnected ? '●' : '○'}
        </span>
      </div>

      <div className={styles.messagesContainer}>
        {messages.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No messages yet...</p>
            <p className={styles.emptyHint}>Be the first to say something!</p>
          </div>
        ) : (
          messages.map((message, index) => (
            <div key={`${message.timestamp}-${index}`} className={styles.message}>
              <div className={styles.messageHeader}>
                <span className={styles.messageUsername}>{message.username}</span>
                <span className={styles.messageTime}>{formatTime(message.timestamp)}</span>
              </div>
              <div className={styles.messageText}>{message.text}</div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      <form onSubmit={handleSendMessage} className={styles.chatInputForm}>
        <input
          type="text"
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          placeholder="Type a message..."
          className={styles.chatInput}
          maxLength={500}
          disabled={!isConnected}
        />
        <button 
          type="submit" 
          className={styles.sendButton}
          disabled={!inputText.trim() || !isConnected}
        >
          <Send size={18} />
        </button>
      </form>
    </aside>
  );
};
