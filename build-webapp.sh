#!/bin/bash
set -e

# Build script for Worknest webapp without requiring trunk
# Uses wasm-pack which is more commonly available

echo "🔨 Building Worknest webapp..."

# Configuration
BUILD_MODE="${1:-release}"
DIST_DIR="dist"
GUI_DIR="crates/worknest-gui"

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Build WASM using wasm-pack
echo "📦 Building WASM with wasm-pack..."
cd "$GUI_DIR"

if [ "$BUILD_MODE" = "debug" ]; then
    wasm-pack build --target web --dev --out-dir ../../pkg
else
    wasm-pack build --target web --release --out-dir ../../pkg
fi

cd ../..

# Copy built files to dist
echo "📋 Copying files to dist..."
cp pkg/*.js "$DIST_DIR/"
cp pkg/*.wasm "$DIST_DIR/"

# Create index.html
echo "📝 Generating index.html..."
cat > "$DIST_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Worknest - Project and Ticket Management">
    <title>Worknest</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
            background-color: #1a1a1a;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
        }

        #loading {
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            height: 100vh;
            color: #ffffff;
        }

        .spinner {
            border: 4px solid #333;
            border-top: 4px solid #4CAF50;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin-bottom: 20px;
        }

        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }

        #error {
            display: none;
            padding: 20px;
            margin: 20px;
            background-color: #ff4444;
            color: white;
            border-radius: 4px;
            max-width: 600px;
            margin: 20px auto;
        }

        canvas {
            width: 100%;
            height: 100%;
        }
    </style>
</head>
<body>
    <div id="loading">
        <div class="spinner"></div>
        <div>Loading Worknest...</div>
    </div>
    <div id="error"></div>

    <script type="module">
        import init from './worknest_gui.js';

        async function run() {
            try {
                // Initialize WASM module
                // The start() function runs automatically thanks to #[wasm_bindgen(start)]
                await init();

                // Hide loading indicator
                document.getElementById('loading').style.display = 'none';
            } catch (error) {
                console.error('Failed to initialize:', error);
                document.getElementById('loading').style.display = 'none';
                const errorDiv = document.getElementById('error');
                errorDiv.style.display = 'block';
                errorDiv.innerHTML = `
                    <h2>Failed to load application</h2>
                    <p>${error.message}</p>
                    <p>Please check the browser console for more details.</p>
                `;
            }
        }

        run();
    </script>
</body>
</html>
EOF

# Create a simple server script for local development
echo "📝 Creating development server script..."
cat > "$DIST_DIR/serve.py" << 'EOF'
#!/usr/bin/env python3
"""Simple HTTP server for serving the Worknest webapp locally."""
import http.server
import socketserver
import os

PORT = 8080
DIRECTORY = os.path.dirname(os.path.abspath(__file__))

class MyHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def end_headers(self):
        # Add CORS headers for development
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

with socketserver.TCPServer(("", PORT), MyHTTPRequestHandler) as httpd:
    print(f"Serving Worknest at http://localhost:{PORT}")
    print("Press Ctrl+C to stop")
    httpd.serve_forever()
EOF

chmod +x "$DIST_DIR/serve.py"

# Display build info
echo ""
echo "✅ Build complete!"
echo ""
echo "📊 Build Information:"
echo "   Mode: $BUILD_MODE"
echo "   Output: $DIST_DIR/"
echo ""

# Show file sizes
echo "📦 Output files:"
ls -lh "$DIST_DIR" | grep -E '\.(html|js|wasm)$' || true

echo ""
echo "🚀 To serve locally:"
echo "   cd $DIST_DIR && python3 serve.py"
echo "   Then open http://localhost:8080"
echo ""
echo "📤 To deploy:"
echo "   Upload the contents of '$DIST_DIR/' to your web host"
echo ""
