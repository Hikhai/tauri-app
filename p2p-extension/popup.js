// Simple popup for extension status
document.addEventListener('DOMContentLoaded', () => {
  const statusDiv = document.getElementById('status');
  const urlDiv = document.getElementById('current-url');
  
  // Get current tab info
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    const currentTab = tabs[0];
    if (currentTab) {
      urlDiv.textContent = currentTab.url;
      
      // Check if we're on a supported site
      const url = currentTab.url;
      if (url.includes('p2p.binance.com') || url.includes('binance.com')) {
        statusDiv.textContent = '✅ Active on supported site';
        statusDiv.className = 'status active';
      } else {
        statusDiv.textContent = '⏸️ Waiting for Binance P2P';
        statusDiv.className = 'status waiting';
      }
    }
  });
});

// Test WebSocket connection
async function testConnection() {
  const testBtn = document.getElementById('test-btn');
  const resultDiv = document.getElementById('test-result');
  
  testBtn.disabled = true;
  testBtn.textContent = 'Testing...';
  
  try {
    const ws = new WebSocket('ws://127.0.0.1:8123');
    
    const timeout = setTimeout(() => {
      ws.close();
      resultDiv.textContent = '❌ Connection timeout';
      resultDiv.className = 'result error';
    }, 3000);
    
    ws.onopen = () => {
      clearTimeout(timeout);
      ws.send(JSON.stringify({
        kind: 'TEST',
        payload: { test: true, timestamp: Date.now() }
      }));
      resultDiv.textContent = '✅ Connected to Tauri app';
      resultDiv.className = 'result success';
      ws.close();
    };
    
    ws.onerror = () => {
      clearTimeout(timeout);
      resultDiv.textContent = '❌ Cannot connect to Tauri app';
      resultDiv.className = 'result error';
    };
    
  } catch (error) {
    resultDiv.textContent = '❌ Connection failed';
    resultDiv.className = 'result error';
  } finally {
    testBtn.disabled = false;
    testBtn.textContent = 'Test Connection';
  }
}

document.getElementById('test-btn')?.addEventListener('click', testConnection);