(function() {
  'use strict';
  const VERSION = '0.2.1';
  const DEBUG = false; // set true for limited console diagnostics
  
  // Enhanced target patterns with more specific matching
  const TARGET_PATTERNS = [
    '/bapi/c2c/',
    '/gateway-api/v1/private/c2c/',
    '/sapi/v1/c2c/',
    '/c2c/'
  ];

  const RELEVANT_KEYWORDS = [
    'order', 'pay', 'match', 'trade', 'detail', 
    'getOrderDetail', 'getPayMethod', 'orderStatus'
  ];

  // Debounce để tránh spam
  const sentUrls = new Set();
  const DEBOUNCE_TIME = 1000;

  function isTarget(url) {
    if (typeof url !== 'string') return false;
    const lower = url.toLowerCase();
    const hasPattern = TARGET_PATTERNS.some(p => lower.includes(p));
    if (!hasPattern) return false;
    const hasKeyword = RELEVANT_KEYWORDS.some(k => lower.includes(k));
    return hasKeyword;
  }

  function shouldSend(url) {
    const key = `${url}_${Date.now().toString().slice(0, -3)}`; // Round to seconds
    if (sentUrls.has(key)) return false;
    
    sentUrls.add(key);
    // Clean old entries
    if (sentUrls.size > 50) {
      const oldEntries = Array.from(sentUrls).slice(0, 25);
      oldEntries.forEach(entry => sentUrls.delete(entry));
    }
    return true;
  }

  let diagCount = 0; // only used when DEBUG
  function emit(obj) {
    try {
      if (!shouldSend(obj.url)) return;
      if (DEBUG && diagCount < 10) {
        diagCount++;
        console.log('[P2P Inject] Capture', obj.method || obj.type, obj.url, 'status=', obj.status);
      }
      window.postMessage({
        __P2P_CAPTURE__: {
          ...obj,
          userAgent: navigator.userAgent.slice(0, 80),
          timestamp: new Date().toISOString(),
          version: VERSION
        }
      }, '*');
    } catch (error) {
      console.error('[P2P Inject] Emit error:', error);
    }
  }

  // Enhanced fetch hook
  const originalFetch = window.fetch;
  window.fetch = async function(...args) {
    const startTime = Date.now();
    
    try {
      const response = await originalFetch.apply(this, args);
      
      let url = args[0];
      if (url && typeof url === 'object' && 'url' in url) {
        url = url.url;
      }
      
      if (isTarget(url)) {
        const clone = response.clone();
        clone.json()
          .then(data => {
            emit({
              type: 'fetch',
              url,
              method: args[1]?.method || 'GET',
              status: response.status,
              duration: Date.now() - startTime,
              data
            });
          })
          .catch(error => {
            console.warn('[P2P Inject] JSON parse failed for:', url, error);
          });
      }
      
      return response;
    } catch (error) {
      console.error('[P2P Inject] Fetch error:', error);
      throw error;
    }
  };

  // Enhanced XHR hook
  const OriginalXHR = window.XMLHttpRequest;
  function PatchedXHR() {
    const xhr = new OriginalXHR();
    let requestUrl = '';
    let requestMethod = 'GET';
    let startTime = 0;
    
    const openOriginal = xhr.open;
    xhr.open = function(method, url, ...rest) {
      requestUrl = url;
      requestMethod = method;
      startTime = Date.now();
      return openOriginal.call(xhr, method, url, ...rest);
    };
    
    xhr.addEventListener('load', function() {
      try {
        if (isTarget(requestUrl)) {
          if (xhr.responseType === '' || xhr.responseType === 'text') {
            const responseText = xhr.responseText;
            if (responseText && responseText.trim().startsWith('{')) {
              try {
                const data = JSON.parse(responseText);
                emit({
                  type: 'xhr',
                  url: requestUrl,
                  method: requestMethod,
                  status: xhr.status,
                  duration: Date.now() - startTime,
                  data
                });
              } catch (parseError) {
                console.warn('[P2P Inject] XHR JSON parse failed:', parseError);
              }
            }
          }
        }
      } catch (error) {
        console.error('[P2P Inject] XHR processing error:', error);
      }
    });
    
    return xhr;
  }
  
  // Copy prototype methods
  Object.setPrototypeOf(PatchedXHR.prototype, OriginalXHR.prototype);
  Object.setPrototypeOf(PatchedXHR, OriginalXHR);
  
  window.XMLHttpRequest = PatchedXHR;

  if (DEBUG) console.log('[P2P Inject] Hook installed -', VERSION);

  // Optional passive logging (disabled in prod) could be re-added here if needed.
})();