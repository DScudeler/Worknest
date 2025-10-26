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

2. **wasm-pack** (WASM build tool)
   ```bash
   cargo install wasm-pack
   ```

3. **Python 3** (for local development server)
   - Usually pre-installed on macOS and Linux
   - Windows: Download from [python.org](https://www.python.org/)

## Building for Web

### Quick Start

```bash
# Build and serve the webapp (one command!)
make serve-webapp

# Or build only
make build-webapp
```

### Development Build

```bash
# Build in debug mode (faster builds)
./build-webapp.sh debug

# Serve locally
cd dist && python3 serve.py
```

This will:
- Build the WASM application using wasm-pack
- Generate optimized JavaScript glue code
- Create a complete `dist/` directory ready to serve
- Start a development server at `http://localhost:8080`

### Production Build

```bash
# Build optimized WASM bundle
./build-webapp.sh release

# Or use make
make build-webapp
```

The output will be in the `dist/` directory:
- `index.html` - Main HTML file with loading UI
- `worknest_gui.js` - JavaScript module
- `worknest_gui_bg.wasm` - Compiled WebAssembly binary
- `serve.py` - Local development server

### Build Options

```bash
# Debug build (faster, larger file size)
./build-webapp.sh debug

# Release build (slower, optimized and smaller)
./build-webapp.sh release

# Clean build artifacts
make clean
# Or manually:
rm -rf dist pkg target
```

## Demo Mode vs Backend API

### Demo Mode (Default)

**The webapp runs in full demo mode by default** - no backend required!

- All data stored in-memory and browser localStorage
- Full CRUD operations on projects and tickets
- Perfect for testing, development, and demonstrations
- Data persists across page reloads (via localStorage)
- Notifications indicate "Demo Mode" operations

Simply build and serve the webapp - it works standalone!

### Backend API (Optional)

For production use with persistent database storage:

```bash
# Start the API server
cargo run -p worknest-api
```

By default, the API server runs on `http://localhost:3000`.

### Configure API Endpoint

The web app automatically detects the API URL from `window.location.origin`. For development with a different API server:

1. Update the API URL in `crates/worknest-gui/src/api_client.rs`
2. Or use a reverse proxy to serve both the WASM app and API from the same origin
3. Uncomment API calls in screen files (currently marked with TODO comments)

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

Currently uses demo mode by default. For production API integration:

1. Update `crates/worknest-gui/src/api_client.rs` to set the API URL
2. Uncomment API calls in screen files (replace demo operations)
3. Configure CORS on your backend server

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
   ./build-webapp.sh release
   # or
   make build-webapp
   ```

2. **Use wasm-opt** (part of binaryen) for further optimization
   ```bash
   # Install binaryen
   cargo install wasm-opt

   # Optimize WASM file
   wasm-opt dist/*.wasm -O3 -o dist/optimized.wasm
   ```

3. **LTO is already enabled** in `Cargo.toml`:
   ```toml
   [profile.release]
   lto = true
   opt-level = 3
   codegen-units = 1
   strip = true
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
make clean
./build-webapp.sh release

# Or manually:
rm -rf dist pkg target
cargo clean
./build-webapp.sh release
```

## Next Steps

- [ ] Add Progressive Web App (PWA) support
- [ ] Implement service worker for true offline mode
- [ ] Add WebSocket support for real-time updates
- [ ] Optimize bundle size further
- [ ] Add end-to-end tests for web app

## Resources

- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [egui Web Demo](https://www.egui.rs/#demo)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [WebAssembly MDN Guide](https://developer.mozilla.org/en-US/docs/WebAssembly)
