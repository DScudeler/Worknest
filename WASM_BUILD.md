# Building Worknest for Web (WASM)

This document explains how to build and deploy Worknest as a web application using WebAssembly (WASM).

## Architecture

Worknest now supports two deployment modes:

### Native Mode
- **Desktop application** with local SQLite database
- Direct database access through `worknest-db`
- Standalone operation

### Web Mode (WASM)
- **Browser-based application** running as WebAssembly
- Communicates with backend API server via HTTP
- Supports offline mode with local browser storage
- Automatic sync when connection restored

## Prerequisites

### For Web/WASM Build

1. **Rust with WASM target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. **Trunk** (WASM web application bundler)
   ```bash
   cargo install trunk
   ```

3. **wasm-bindgen-cli** (optional, for manual builds)
   ```bash
   cargo install wasm-bindgen-cli
   ```

## Building for Web

### Development Build

```bash
# Build and serve locally with hot reload
trunk serve

# Or specify a custom port
trunk serve --port 8080
```

This will:
- Build the WASM application
- Start a development server at `http://127.0.0.1:8080`
- Watch for changes and auto-rebuild

### Production Build

```bash
# Build optimized WASM bundle
trunk build --release
```

The output will be in the `dist/` directory:
- `index.html` - Main HTML file
- `worknest-*.js` - JavaScript glue code
- `worknest-*.wasm` - Compiled WebAssembly binary
- Static assets

### Build Options

```bash
# Build with custom public URL (for subdirectory deployment)
trunk build --release --public-url /worknest/

# Clean build artifacts
trunk clean
```

## Running the Backend API Server

The web application requires the backend API server to be running:

```bash
# Start the API server
cargo run -p worknest-api
```

By default, the API server runs on `http://localhost:3000`.

### Configure API Endpoint

The web app automatically detects the API URL from `window.location.origin`. For development with a different API server:

1. Update the API URL in `crates/worknest-gui/src/main.rs` (web mode initialization)
2. Or use a reverse proxy to serve both the WASM app and API from the same origin

## Deployment

### Static Hosting

The built WASM application is purely static and can be deployed to any static hosting service:

- **Netlify**: Drop the `dist/` folder
- **Vercel**: Deploy from repository
- **GitHub Pages**: Push `dist/` to gh-pages branch
- **AWS S3 + CloudFront**
- **Nginx/Apache**: Serve the `dist/` directory

### With Backend API

For production, you need:

1. **Frontend (WASM)**: Deploy to static hosting
2. **Backend (API)**: Deploy to a server/cloud platform
3. **Database**: PostgreSQL or MySQL for production (SQLite for dev)

Example nginx configuration:

```nginx
server {
    listen 80;
    server_name worknest.example.com;

    # Serve WASM frontend
    location / {
        root /var/www/worknest/dist;
        try_files $uri $uri/ /index.html;

        # WASM MIME type
        types {
            application/wasm wasm;
        }
    }

    # Proxy API requests to backend
    location /api {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Environment Variables

### Backend (API Server)

```bash
# Database
export WORKNEST_DB_PATH="/path/to/database.db"

# Authentication
export WORKNEST_SECRET_KEY="your-secret-key-here"

# Server
export WORKNEST_HOST="0.0.0.0"
export WORKNEST_PORT="3000"
```

### Frontend (Build Time)

Set via `Trunk.toml` or environment:

```bash
# API base URL for production
export PUBLIC_API_URL="https://api.worknest.example.com"
```

## Features

### Online Mode
- Full API access
- Real-time data sync
- Multi-user collaboration
- Server-side persistence

### Offline Mode
- Local browser storage (IndexedDB)
- Continue working without connection
- Automatic sync when online
- Data cached locally

## Development

### File Structure

```
crates/worknest-gui/
├── src/
│   ├── main.rs          # Entry point (native + web)
│   ├── lib.rs           # Library exports
│   ├── api_client.rs    # HTTP client for API
│   ├── state.rs         # App state (supports both modes)
│   ├── screens/         # UI screens
│   └── theme.rs         # UI theming
├── index.html           # Web entry point
└── Cargo.toml           # Dependencies (conditional)
```

### Conditional Compilation

Code is conditionally compiled based on target:

```rust
#[cfg(not(target_arch = "wasm32"))]
// Native-only code (database access)

#[cfg(target_arch = "wasm32")]
// Web-only code (API client)
```

### Testing

```bash
# Test core logic (not GUI)
cargo test -p worknest-core -p worknest-db -p worknest-auth -p worknest-api

# All tests
cargo test --workspace
```

## Browser Support

Worknest WASM app supports:
- Chrome/Edge 90+
- Firefox 87+
- Safari 15.4+
- Opera 76+

Requirements:
- WebAssembly support
- ES6 modules
- WebGL 2.0 (for rendering)

## Performance

### WASM Binary Size

- Development: ~5-10 MB
- Production (optimized): ~2-3 MB
- Compressed (gzip): ~500 KB - 1 MB

### Optimization Tips

1. **Enable release mode**
   ```bash
   trunk build --release
   ```

2. **Use wasm-opt** (part of binaryen)
   ```bash
   wasm-opt dist/*.wasm -O3 -o dist/optimized.wasm
   ```

3. **Enable LTO** in `Cargo.toml`:
   ```toml
   [profile.release]
   lto = true
   opt-level = "z"  # Optimize for size
   ```

## Troubleshooting

### CORS Issues

If the web app can't connect to the API:

```javascript
// Backend must allow CORS
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE
Access-Control-Allow-Headers: Content-Type, Authorization
```

The `worknest-api` server has CORS enabled by default in development mode.

### WASM Load Failures

1. Ensure MIME type is correct: `application/wasm`
2. Check browser console for errors
3. Verify server is serving WASM files correctly

### Build Errors

```bash
# Clean and rebuild
trunk clean
cargo clean
trunk build
```

## Next Steps

- [ ] Add Progressive Web App (PWA) support
- [ ] Implement service worker for true offline mode
- [ ] Add WebSocket support for real-time updates
- [ ] Optimize bundle size further
- [ ] Add end-to-end tests for web app

## Resources

- [Trunk Documentation](https://trunkrs.dev/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [egui Web Demo](https://www.egui.rs/#demo)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
