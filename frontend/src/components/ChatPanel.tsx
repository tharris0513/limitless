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
  const reconnectTimeoutRef = useRef<number | undefined>(undefined);
  const reconnectAttemptsRef = useRef(0);
  const shouldReconnectRef = useRef(true);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    // Reset reconnection flag on mount
    shouldReconnectRef.current = true;
    
    // Use the API URL from environment for both dev and prod
    const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:8080';
    const apiHost = new URL(apiUrl).hostname;
    const apiProtocol = new URL(apiUrl).protocol === 'https:' ? 'wss:' : 'ws:';
    const apiPort = new URL(apiUrl).port;
    
    // Build WebSocket URL using API domain
    const wsUrl = apiPort 
      ? `${apiProtocol}//${apiHost}:${apiPort}/api/chat/ws`
      : `${apiProtocol}//${apiHost}/api/chat/ws`;

    let pingInterval: number | undefined;
    let isCleanedUp = false; // Track if cleanup has been called

    const connectWebSocket = () => {
      // Don't connect if already cleaned up (prevents double connection in StrictMode)
      if (isCleanedUp) {
        console.log('Skipping connection - component unmounted');
        return;
      }

      console.log('Connecting to WebSocket:', wsUrl);

      // Connect to WebSocket
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        if (isCleanedUp) {
          ws.close();
          return;
        }
        console.log('WebSocket connected');
        setIsConnected(true);
        reconnectAttemptsRef.current = 0; // Reset reconnection attempts on successful connection

        // Send ping every 30 seconds to keep connection alive
        pingInterval = window.setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            console.log('Sending keepalive ping');
            ws.send(JSON.stringify({ type: 'ping' }));
          }
        }, 30000);
      };

      ws.onmessage = (event) => {
        if (isCleanedUp) return; // Ignore messages after cleanup
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

        // Attempt to reconnect with exponential backoff
        if (shouldReconnectRef.current && !isCleanedUp) {
          reconnectAttemptsRef.current++;
          const delay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 30000); // Max 30 seconds
          console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttemptsRef.current})...`);
          
          reconnectTimeoutRef.current = window.setTimeout(() => {
            connectWebSocket();
          }, delay);
        }
      };

      wsRef.current = ws;
    };

    // Initial connection
    connectWebSocket();

    // Cleanup on unmount
    return () => {
      console.log('ChatPanel cleanup - closing WebSocket');
      isCleanedUp = true;
      shouldReconnectRef.current = false; // Stop reconnection attempts
      if (pingInterval) {
        clearInterval(pingInterval);
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
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

      // Send to WebSocket server (server will broadcast it back to all clients including us)
      wsRef.current.send(JSON.stringify(message));
      
      // Clear input immediately for better UX
      // Note: We don't add the message locally - the server will broadcast it back to us
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
