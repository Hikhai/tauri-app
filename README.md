# P2P Assistant (Tauri + SvelteKit)

Local desktop helper to capture and display Binance P2P order list & payment info using a companion browser extension and a lightweight Rust WebSocket server.

## Architecture
* Browser Extension (MV3) injects hooks for fetch / XHR to capture relevant C2C endpoints.
* Extension forwards compact JSON payloads via `ws://127.0.0.1:8123` to the Tauri app.
* Rust backend parses flexible order list / order detail schemas, stores in-memory.
* SvelteKit frontend polls Tauri commands to render the order table with payment fields.

## Key Features
* Order list & detail auto aggregation (supports multiple response nesting patterns).
* Displays bank name, account number, account holder, branch (sub bank) and transfer content (always set = order number for reliability).
* Simple dedup window to reduce spam.

## Development
Install deps first:
```bash
npm install
```
Run app (dev):
```bash
npm run tauri:dev
```

Load `p2p-extension` into Chromium (Developer Mode > Load unpacked).

## Phase 3 Cleanup
Diagnostics removed / gated behind `DEBUG` flags in extension scripts.
Force detail helper & passive noisy logs removed.

## Next Ideas
* Persistent storage (SQLite or JSON snapshot) for history.
* Friendly status code mapping.
* Optional raw instruction field preservation.
* SPA route mutation observer for more robust capture on dynamic navigations.

## Security / Privacy
All processing is local. No external network calls besides Binance endpoints already executed by the browser session.

## License
Private / internal (add a proper license if distributing).
