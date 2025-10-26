# Deployment Guide

This document describes how to deploy Worknest webapp to various hosting platforms.

## Table of Contents

- [GitHub Pages (Recommended for Demo)](#github-pages)
- [Netlify](#netlify)
- [Vercel](#vercel)
- [Static Hosting (nginx, Apache, S3, etc.)](#static-hosting)

---

## GitHub Pages

**Best for:** Free hosting of the demo mode directly from your GitHub repository.

### Prerequisites

- GitHub repository with the Worknest code
- Repository permissions to enable GitHub Pages

### Setup Instructions

1. **Enable GitHub Pages:**
   ```
   Repository Settings → Pages → Source: "GitHub Actions"
   ```

2. **Deploy automatically on push to main:**
   - The workflow `.github/workflows/deploy-pages.yml` is already configured
   - Every push to `main` branch triggers a deployment
   - Wait 2-5 minutes for the first deployment

3. **Access your demo:**
   ```
   https://YOUR_USERNAME.github.io/Worknest/
   ```

   For this repository: `https://dscudeler.github.io/Worknest/`

4. **Manual deployment (optional):**
   - Go to Actions tab in GitHub
   - Select "Deploy to GitHub Pages" workflow
   - Click "Run workflow" button

### How It Works

The GitHub Pages workflow:
1. Checks out the code
2. Installs Rust and wasm-pack
3. Builds the webapp with `./build-webapp.sh release`
4. Uploads the `dist/` folder to GitHub Pages
5. Deploys to your GitHub Pages URL

### Custom Domain (Optional)

1. Add a `CNAME` file to the `dist/` directory with your domain
2. Configure DNS records:
   ```
   CNAME www.yourdomain.com YOUR_USERNAME.github.io
   ```
3. Update workflow to preserve CNAME file during build

---

## Netlify

**Best for:** Automatic deployments with preview URLs for pull requests.

### Setup Instructions

1. **Connect Repository:**
   - Sign in to [Netlify](https://netlify.com)
   - Click "New site from Git"
   - Connect your GitHub repository

2. **Build Settings:**
   ```
   Build command: ./build-webapp.sh release
   Publish directory: dist
   ```

3. **Add Build Image:**
   - Go to Site Settings → Build & Deploy → Build Image
   - Select "Ubuntu Focal 20.04"

4. **Environment Variables (if needed):**
   - No environment variables required for demo mode
   - For API integration, set `API_URL` environment variable

5. **Deploy:**
   - Netlify automatically deploys on every push to main
   - Preview deployments for pull requests

### Custom Domain

1. Go to Site Settings → Domain management
2. Add your custom domain
3. Netlify provides SSL certificate automatically

---

## Vercel

**Best for:** Fast global CDN and automatic HTTPS.

### Setup Instructions

1. **Import Project:**
   - Sign in to [Vercel](https://vercel.com)
   - Click "New Project"
   - Import from GitHub

2. **Build Settings:**
   ```
   Framework Preset: Other
   Build Command: ./build-webapp.sh release
   Output Directory: dist
   Install Command: curl https://sh.rustup.rs -sSf | sh -s -- -y &&
                    source $HOME/.cargo/env &&
                    rustup target add wasm32-unknown-unknown &&
                    cargo install wasm-pack
   ```

3. **Deploy:**
   - Vercel auto-deploys on push to main
   - Preview URLs for branches and PRs

### Custom Domain

1. Go to Project Settings → Domains
2. Add your domain
3. Configure DNS as instructed

---

## Static Hosting

**Best for:** Deploying to your own infrastructure (nginx, Apache, AWS S3, etc.)

### Build Locally

```bash
# Build the webapp
./build-webapp.sh release

# The output is in dist/
ls -lh dist/
```

### Deploy Files

Upload all files from `dist/` directory to your web server:
- `index.html`
- `worknest_gui.js`
- `worknest_gui_bg.wasm`
- `serve.py` (optional, for local testing)

### nginx Configuration

```nginx
server {
    listen 80;
    server_name worknest.example.com;

    root /var/www/worknest;
    index index.html;

    # WASM MIME type
    types {
        application/wasm wasm;
    }

    # Serve static files
    location / {
        try_files $uri $uri/ /index.html;

        # CORS headers (if API is on different domain)
        add_header Access-Control-Allow-Origin *;
        add_header Cross-Origin-Opener-Policy same-origin;
        add_header Cross-Origin-Embedder-Policy require-corp;
    }

    # Cache WASM files
    location ~* \.wasm$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # SSL configuration (recommended)
    # listen 443 ssl;
    # ssl_certificate /path/to/cert.pem;
    # ssl_certificate_key /path/to/key.pem;
}
```

### Apache Configuration

```apache
<VirtualHost *:80>
    ServerName worknest.example.com
    DocumentRoot /var/www/worknest

    # WASM MIME type
    AddType application/wasm .wasm

    # Enable CORS
    Header set Access-Control-Allow-Origin "*"
    Header set Cross-Origin-Opener-Policy "same-origin"
    Header set Cross-Origin-Embedder-Policy "require-corp"

    # Cache WASM files
    <FilesMatch "\.wasm$">
        Header set Cache-Control "public, max-age=31536000, immutable"
    </FilesMatch>

    # Fallback to index.html
    <Directory /var/www/worknest>
        Options -Indexes +FollowSymLinks
        AllowOverride All
        Require all granted

        RewriteEngine On
        RewriteBase /
        RewriteRule ^index\.html$ - [L]
        RewriteCond %{REQUEST_FILENAME} !-f
        RewriteCond %{REQUEST_FILENAME} !-d
        RewriteRule . /index.html [L]
    </Directory>
</VirtualHost>
```

### AWS S3 + CloudFront

1. **Create S3 Bucket:**
   ```bash
   aws s3 mb s3://worknest-demo
   aws s3 sync dist/ s3://worknest-demo/ --acl public-read
   ```

2. **Configure Bucket for Static Hosting:**
   - Enable static website hosting
   - Set index document: `index.html`
   - Set error document: `index.html`

3. **Set MIME Types:**
   - Ensure `.wasm` files have `application/wasm` content type
   ```bash
   aws s3 cp dist/ s3://worknest-demo/ --recursive \
       --exclude "*" --include "*.wasm" \
       --content-type="application/wasm" \
       --metadata-directive REPLACE
   ```

4. **Create CloudFront Distribution:**
   - Origin: Your S3 bucket
   - Default root object: `index.html`
   - Enable compression
   - Custom error responses: 404 → /index.html

---

## Demo Mode vs Production

### Demo Mode (Current)

- **No backend required**
- Data stored in browser localStorage
- Perfect for testing and demonstrations
- All changes are local to the browser

### Production (Future)

To deploy with a real backend:

1. Deploy the backend API (worknest-api crate) to a server
2. Update `api_client.rs` with production API URL
3. Uncomment API calls in screen files
4. Configure CORS on the backend
5. Deploy both frontend and backend

See [ARCHITECTURE.md](./ARCHITECTURE.md) for backend deployment details.

---

## Troubleshooting

### WASM Not Loading

**Problem:** Browser console shows WASM loading errors

**Solution:**
- Ensure MIME type is set to `application/wasm`
- Check CORS headers are configured
- Verify WASM file is being served correctly

### Blank Page

**Problem:** Page loads but shows nothing

**Solution:**
- Check browser console for JavaScript errors
- Ensure all files (HTML, JS, WASM) are uploaded
- Clear browser cache and reload

### 404 on Refresh

**Problem:** Refreshing page shows 404 error

**Solution:**
- Configure server to serve `index.html` for all routes
- Use appropriate rewrite rules (see nginx/Apache configs above)

---

## Performance Optimization

### Compression

Enable gzip/brotli compression on your server:

```nginx
# nginx
gzip on;
gzip_types application/wasm application/javascript text/html;
gzip_min_length 1000;

# Or use brotli for better compression
brotli on;
brotli_types application/wasm application/javascript;
```

### CDN

Use a CDN for global distribution:
- CloudFlare
- AWS CloudFront
- Fastly
- Netlify (built-in)
- Vercel (built-in)

### Caching

Set cache headers for static assets:
```
Cache-Control: public, max-age=31536000, immutable
```

---

## Security

### HTTPS

Always use HTTPS in production:
- Free SSL certificates: [Let's Encrypt](https://letsencrypt.org/)
- Most hosting platforms provide automatic HTTPS

### Content Security Policy

Add CSP headers for security:

```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline';";
```

### CORS

Configure CORS only if API is on a different domain:

```nginx
add_header Access-Control-Allow-Origin "https://api.yourdomain.com";
add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE";
add_header Access-Control-Allow-Headers "Content-Type, Authorization";
```

---

## Monitoring

### GitHub Pages

- Monitor via GitHub Actions
- Check deployment status in Actions tab

### Netlify/Vercel

- Built-in analytics and monitoring
- Real-time logs in dashboard

### Custom Hosting

Recommended monitoring tools:
- **Uptime:** UptimeRobot, Pingdom
- **Analytics:** Plausible, Google Analytics
- **Error Tracking:** Sentry
- **Performance:** Lighthouse CI

---

## Questions?

For deployment issues, please:
1. Check the [Troubleshooting](#troubleshooting) section
2. Review [GitHub Discussions](https://github.com/DScudeler/Worknest/discussions)
3. Open an issue on GitHub
