(function() {
  'use strict';
  
  const WS_URL = 'ws://127.0.0.1:8123';
  const MAX_RECONNECT_ATTEMPTS = 10;
  const INITIAL_RECONNECT_DELAY = 1000;
  const MAX_RECONNECT_DELAY = 30000;
  
  let ws;
  let reconnectAttempts = 0;
  let reconnectDelay = INITIAL_RECONNECT_DELAY;
  let isInjected = false;
  let messageQueue = [];
  const MAX_QUEUE_SIZE = 100;

  function injectScript() {
    if (isInjected) return;
    
    try {
      const script = document.createElement('script');
      script.src = chrome.runtime.getURL('injected.js');
      script.async = false;
      script.onload = () => {
        script.remove();
        isInjected = true;
        console.log('[P2P Ext] Injected script loaded successfully');
      };
      script.onerror = () => {
        console.error('[P2P Ext] Failed to load injected script');
      };
      
      (document.head || document.documentElement).appendChild(script);
    } catch (error) {
      console.error('[P2P Ext] Script injection failed:', error);
    }
  }

  function connectWS() {
    if (ws && ws.readyState === WebSocket.CONNECTING) {
      return; // Already connecting
    }
    
    try {
      ws = new WebSocket(WS_URL);
      
      ws.onopen = () => {
        console.log('[P2P Ext] WebSocket connected successfully');
        reconnectAttempts = 0;
        reconnectDelay = INITIAL_RECONNECT_DELAY;
        
        // Send queued messages
        while (messageQueue.length > 0 && ws.readyState === WebSocket.OPEN) {
          const message = messageQueue.shift();
          ws.send(message);
        }
      };
      
      ws.onclose = (event) => {
        console.log(`[P2P Ext] WebSocket closed (code: ${event.code}, reason: ${event.reason})`);
        scheduleReconnect();
      };
      
      ws.onerror = (error) => {
        console.error('[P2P Ext] WebSocket error:', error);
      };
      
    } catch (error) {
      console.error('[P2P Ext] WebSocket connection failed:', error);
      scheduleReconnect();
    }
  }

  function scheduleReconnect() {
    if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
      console.error('[P2P Ext] Max reconnection attempts reached. Stopping.');
      return;
    }
    
    reconnectAttempts++;
    console.log(`[P2P Ext] Reconnecting in ${reconnectDelay}ms (attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`);
    
    setTimeout(() => {
      connectWS();
    }, reconnectDelay);
    
    // Exponential backoff with max limit
    reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY);
  }

  function sendMessage(data) {
    const message = JSON.stringify({
      kind: 'NET_CAPTURE',
      payload: data.__P2P_CAPTURE__,
      extension_version: '0.2.1',
      url: window.location.href,
      timestamp: Date.now()
    });
    
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(message);
    } else {
      // Queue message if not connected
      messageQueue.push(message);
      if (messageQueue.length > MAX_QUEUE_SIZE) {
        messageQueue.shift(); // Remove oldest message
      }
      console.warn('[P2P Ext] WebSocket not ready, message queued');
    }
  }

  // Enhanced message listener
  window.addEventListener('message', (event) => {
    // Security check
    if (event.source !== window) return;
    
    const data = event.data;
    if (!data || !data.__P2P_CAPTURE__) return;
    
    try {
      sendMessage(data);
      console.log('[P2P Ext] Message forwarded:', data.__P2P_CAPTURE__.url);
    } catch (error) {
      console.error('[P2P Ext] Failed to send message:', error);
    }
  });

  // Page visibility change handler
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible' && (!ws || ws.readyState !== WebSocket.OPEN)) {
      console.log('[P2P Ext] Page became visible, checking connection...');
      connectWS();
    }
  });

  // Initialize
  function initialize() {
    console.log('[P2P Ext] Initializing P2P Assistant Bridge v0.2.1');
    injectScript();
    connectWS();
  }

  // Start when DOM is ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initialize);
  } else {
    initialize();
  }
})();