# P2P Assistant Chrome Extension v0.2.1

## ✨ Tính năng mới trong v0.2.1

### 🔧 Cải tiến Extension
- **Enhanced Error Handling**: Xử lý lỗi tốt hơn và logging chi tiết
- **Debouncing**: Tránh spam duplicate requests
- **Queue System**: Message queue khi WebSocket disconnected
- **Reconnection Logic**: Auto-reconnect với exponential backoff
- **Status Popup**: UI popup để monitor trạng thái extension

### 🎯 Pattern Matching Cải tiến
- Hỗ trợ nhiều API endpoints: `/bapi/c2c/`, `/gateway-api/v1/private/c2c/`, `/sapi/v1/c2c/`
- Filter keywords cụ thể: `order`, `pay`, `match`, `trade`, `detail`
- Giảm noise với debouncing và smart filtering

### 📊 Enhanced Data Capture
- Capture thêm metadata: method, status, duration
- User agent và timestamp thông tin
- Error handling cho JSON parsing

## 🚀 Hướng dẫn cài đặt

### 1. Load Extension vào Chrome
1. Mở Chrome và đi tới `chrome://extensions`
2. Bật "Developer mode" (góc trên bên phải)
3. Click "Load unpacked"
4. Chọn thư mục `p2p-extension/` (thư mục này)
5. Extension sẽ xuất hiện với icon P2P Assistant

### 2. Kiểm tra Extension hoạt động
1. **Đảm bảo Tauri app đang chạy**: `npm run tauri:dev`
2. **Click vào extension icon** → popup sẽ hiện status
3. **Mở Binance P2P**: https://p2p.binance.com
4. **Kiểm tra DevTools (F12)**:
   - Console phải thấy: `[P2P Ext] WebSocket connected successfully`
   - Console trang phải thấy: `[P2P Inject] Enhanced hook installed - v0.2.1`

### 3. Test Connection
- Click extension icon → "Test Connection"
- Phải thấy "✅ Connected to Tauri app"
- Nếu lỗi: kiểm tra Tauri app có chạy không

### 4. Capture Real Data
1. **Vào chi tiết order P2P** (click vào một order bất kỳ)
2. **Terminal Tauri sẽ hiện**:
   ```
   [WS] Client connected
   [WS] Received (total stored: X)
   ```
3. **Trong Tauri app**: Click "Refresh" → xem JSON messages
4. **Console extension**: Thấy `[P2P Inject] Capturing: https://p2p.binance.com/bapi/...`

## 📁 Cấu trúc Files

```
p2p-extension/
├── manifest.json     # Extension configuration (MV3)
├── content.js       # Content script + WebSocket bridge
├── injected.js      # Hook fetch/XHR trong page context  
├── popup.html       # Extension popup UI
├── popup.js         # Popup logic và connection test
└── README.md        # Tài liệu này
```

## 🔧 Troubleshooting

### ❌ "WebSocket connection failed"
**Nguyên nhân**: Tauri app chưa chạy hoặc port bị chặn
**Giải pháp**:
- Chạy `npm run tauri:dev` 
- Kiểm tra log có `[WS] Binding 127.0.0.1:8123`
- Thử đổi port trong cả extension và Tauri nếu bị conflict

### ❌ "Hook not installed" 
**Nguyên nhân**: Script injection thất bại
**Giải pháp**:
- Reload trang Binance P2P (Ctrl+R)
- Kiểm tra extension có được load đúng trong chrome://extensions
- Clear browser cache nếu cần

### ❌ "No requests captured"
**Nguyên nhân**: Chưa trigger đúng API calls
**Giải pháp**:
- **Phải vào chi tiết order** (không chỉ list)
- Click vào order number hoặc "View details"
- Kiểm tra Network tab có requests chứa `/bapi/c2c/` không
- Thử refresh trang order detail

### ❌ "JSON parse errors"
**Nguyên nhân**: Response không phải JSON hoặc bị corrupt
**Giải pháp**:
- Bình thường, chỉ cần ignore
- Extension chỉ capture JSON responses
- Non-JSON responses sẽ bị skip

## 🧪 Testing

### Test script có sẵn:
```bash
node test-extension.mjs
```

### Manual test trong DevTools:
```javascript
// Test WebSocket connection
const ws = new WebSocket('ws://127.0.0.1:8123');
ws.onopen = () => ws.send(JSON.stringify({
  kind: 'TEST', 
  payload: {test: true, timestamp: Date.now()}
}));
```

## 📊 Monitored Endpoints

Extension tự động bắt các API calls:
- `*/bapi/c2c/*order*` - Order details
- `*/bapi/c2c/*pay*` - Payment methods  
- `*/bapi/c2c/*match*` - Trade matching
- `*/gateway-api/v1/private/c2c/*` - Alternative API
- `*/sapi/v1/c2c/*` - Public API

## 🔄 Next Steps

Khi extension hoạt động tốt:
1. **Capture được JSON data** chứa order details
2. **Ready cho Pha 3**: Parse data trong Rust
3. **Implement OrderStore**: Lưu trữ và hiển thị bảng
4. **Copy functionality**: Format data để copy

## 📈 Performance Features

- **Debouncing**: Tránh duplicate captures trong 1 giây
- **Queue Management**: Max 100 messages queue
- **Memory Cleanup**: Auto-clean old URL cache
- **Exponential Backoff**: Smart reconnection strategy
- **Selective Injection**: Chỉ inject trên target sites

---

**Version**: 0.2.1  
**Compatible**: Chrome Extensions MV3  
**Target**: Binance P2P (https://p2p.binance.com)