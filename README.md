# EdgeLink Template

Fast, secure URL redirect service and QR code generator built on Cloudflare Workers.

**Try it live:** [isqr.me/edgelink](https://isqr.me/edgelink)

Deploy your own instance in minutes with this template repository.

## Features

- **Minimal JavaScript** - HTML forms use native POST/redirect; JavaScript only for copy-to-clipboard on success page
- **Custom Short URLs** - Optional custom names or auto-generated lowercase IDs
- **QR Code Generation** - SVG QR codes for every shortened URL
- **Smart URL Handling** - Auto-prepends https:// if protocol omitted
- **Secure** - Domain validation, open redirect prevention, safe header handling
- **Collision-Free** - Automatic ID collision detection and retry
- **Edge Performance** - Runs on Cloudflare's global network

## Tech Stack

- **Runtime**: Cloudflare Workers (Rust + WASM)
- **Storage**: Cloudflare KV
- **Build**: `worker-build`
- **Dependencies**: `worker`, `qrcode`, `serde`, `serde_json`

## Setup

1. **Clone and install**
   ```bash
   git clone https://github.com/PeterMHammond/edgelink-template.git
   cd edgelink-template
   ```

2. **Deploy**
   ```bash
   wrangler deploy
   ```

   The KV namespace will be automatically created on first deployment.

3. **Secure with [Zero Trust](https://developers.cloudflare.com/cloudflare-one/applications/configure-apps/) (Strongly Recommended)**

   Protect the `/create` endpoint with Cloudflare Access to prevent unauthorized URL creation. Since the root path `/` redirects to `/create`, this effectively secures the entire application.

   **Steps to configure:**

   1. Navigate to Cloudflare dashboard → **Zero Trust** → **Access** → **Applications**
   2. Click **Add an application** → Select **Self-hosted**
   3. Set **Application domain** to your Worker domain (e.g., `isqr.me`)
   4. Set **Path** to `/create`
   5. Configure **Authentication** method (email, Google, GitHub, SSO, etc.)
   6. Create an **Access Policy** to define who can create short URLs
   7. Save and deploy

   **Why this matters:** Without Zero Trust protection, anyone can create short URLs on your domain, potentially leading to abuse or unauthorized usage.

## Development

```bash
# Run locally
wrangler dev

# Build
wrangler deploy --dry-run
```

## Usage

1. Visit `/create`
2. Enter a URL (e.g., `cloudflare.com` or `https://cloudflare.com` - https:// auto-added if omitted)
3. Optionally enter a custom short name (2-20 chars - auto-normalized to lowercase, spaces→hyphens)
4. Get a shortened URL with QR code showing both short URL and target
5. Share the short URL - redirects automatically

**URL Requirements:**
- Must include a domain extension (e.g., `.com`, `.org`, `.io`)
- Protocol (https://) is auto-prepended if not provided
- Examples: `cloudflare.com`, `https://cloudflare.com/login`, `https://developers.cloudflare.com/index.html`

**Custom Name Auto-Normalization:**
- Uppercase → lowercase: `MyLink` becomes `mylink`
- Spaces → hyphens: `my link` becomes `my-link`
- Invalid characters removed: `test_123!` becomes `test-123`

## Architecture

Modular design with clean separation of concerns:
- **`src/lib.rs`** - Minimal router setup (16 lines)
- **`src/routes/`** - Individual route handlers (home→create redirect, create form/handler, redirect validator, custom 404)
- **Minimal JavaScript** - Server-side rendering with HTML forms; JavaScript only for clipboard operations
- **Security-first** - URL validation, domain checking, open redirect prevention

## Acknowledgements

If you're looking for an external URL shortening service, check out [shorturl.com](https://shorturl.com/). EdgeLink is designed for those who want to host their own redirect service on Cloudflare Workers.

## License

MIT
