(function() {
  'use strict';
  // Lightweight production bridge (phase3 cleanup)
  const BRIDGE_VERSION = '0.2.1';
  const DEBUG = false; // flip to true for extra queue / send logs
  
  const WS_URL = 'ws://127.0.0.1:8123';
  const MAX_RECONNECT_ATTEMPTS = 10;
  const INITIAL_RECONNECT_DELAY = 1000;
  const MAX_RECONNECT_DELAY = 30000;
  
  let ws;
  let reconnectAttempts = 0;
  let reconnectDelay = INITIAL_RECONNECT_DELAY;
  let isInjected = false;
  let messageQueue = [];
  let wsConnecting = false; // Thêm flag để tránh spam connection
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
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
      return; // Already connected or connecting
    }
    
    if (wsConnecting) {
      return; // Already in connecting process
    }
    
    wsConnecting = true;
    
    try {
      ws = new WebSocket(WS_URL);
      
      ws.onopen = () => {
        console.log('[P2P Ext] WebSocket connected successfully');
        wsConnecting = false;
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
        wsConnecting = false;
        scheduleReconnect();
      };
      
      ws.onerror = (error) => {
        console.error('[P2P Ext] WebSocket error:', error);
        wsConnecting = false;
      };
      
    } catch (error) {
      console.error('[P2P Ext] WebSocket connection failed:', error);
      wsConnecting = false;
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
    const payload = data.__P2P_CAPTURE__;
    if (!payload) {
      if (DEBUG) console.warn('[P2P Ext] Missing __P2P_CAPTURE__ on data');
      return;
    }
    const messageObj = {
      kind: 'NET_CAPTURE',
      payload,
      extension_version: BRIDGE_VERSION,
      page_url: window.location.href,
      ts_send: Date.now()
    };
    const message = JSON.stringify(messageObj);

    if (ws && ws.readyState === WebSocket.OPEN) {
      try {
        ws.send(message);
        if (DEBUG) console.log('[P2P Ext] Sent ->', payload.url);
      } catch (e) {
        if (DEBUG) console.error('[P2P Ext] Send failed, queueing', e);
        messageQueue.push(message);
      }
    } else {
      messageQueue.push(message);
      if (messageQueue.length > MAX_QUEUE_SIZE) messageQueue.shift();
      if (DEBUG) console.debug('[P2P Ext] Queue size', messageQueue.length, 'waiting WS for', payload.url);
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
      if (DEBUG) console.log('[P2P Ext] Message forwarded:', data.__P2P_CAPTURE__.url);
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
    if (DEBUG) console.log('[P2P Ext] Initializing bridge', BRIDGE_VERSION, 'state=', document.readyState);
    injectScript();
    connectWS();
  }

  // Run immediately (document_start) to avoid missing early requests
  initialize();
})();