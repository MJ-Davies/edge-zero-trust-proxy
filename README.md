# Edge Zero Trust Proxy

An edge-native, identity-aware reverse proxy and security firewall built in Rust and deployed on Cloudflare Workers. The proxy intercepts incoming traffic, cryptographically validates Cloudflare Access Zero Trust JWT assertions via JWKS, evaluates request payloads in real-time using Cloudflare Workers AI (Llama model), and safely proxies verified traffic to upstream public service.

## Architecture & Request Flow
```
[ Client / Postman / Browser ]
│
▼
[ Cloudflare Access ]  ── (Enforces Identity Perimeter & Injects JWT via Cloudflare Zero Trust)
│
▼
[ Rust Proxy Worker ]
├── 1. Cryptographic Identity Verification (JWKS & Cf-Access-Jwt-Assertion via Cloudflare Zero Trust)
├── 2. Edge AI Firewall (Payload Threat Inspection via Workers AI)
└── 3. Sanitized Reverse Proxy Subrequest
│
▼
[ Upstream Backend ]
```

1. **Edge Perimeter:** Cloudflare Access intercepts the request and handles user authentication, injecting a cryptographically signed `Cf-Access-Jwt-Assertion` header.
2. **Cryptographic Validation:** The proxy retrieves the JSON Web Key Set (JWKS) from Cloudflare's public certs endpoint to verify token signature, audience tag (`aud`), and issuer claims.
3. **AI Threat Inspection:** POST/PUT request payloads are extracted and analyzed at the edge using Workers AI to classify incoming payloads as `SAFE` or `UNSAFE` against SQLi, XSS, prompt injection attacks, and other web-based attacks.
4. **Upstream Forwarding:** Validated requests are stripped of transport-level hop headers and forwarded to the configured upstream service.

## Tech Stack

* **Language:** Rust (compiled to `wasm32-unknown-unknown`)
* **Runtime:** Cloudflare Workers (`worker-rs`, V8 / Miniflare)
* **Identity & Access Management:** Cloudflare Access Zero Trust, JWT (`jsonwebtoken`, `rust_crypto`)
* **Security & AI:** Cloudflare Workers AI (`@cf/meta/llama-4-scout-17b-16e-instruct`)
* **Tooling:** Wrangler CLI, `cloudflared`, Cargo, WSL2

## Configuration

Set the required environment variables in your `wrangler.toml`:

```toml
name = "edge-zero-trust-proxy"
main = "build/index.js"
compatibility_date = "2026-08-15"
compatibility_flags = ["global_fetch_strictly_public"]

[build]
command = "cargo install -q \"worker-build@^0.8\" && worker-build --release"

[ai]
binding = "AI"
remote = true

[vars]
UPSTREAM_URL = "[https://mock-backend.your-domain.workers.dev](https://mock-backend.your-domain.workers.dev)"
AI_MODEL = "@cf/meta/llama-4-scout-17b-16e-instruct" # Or any other available model
TEAM_DOMAIN = "https://<your-team>.cloudflareaccess.com"
AUD_TAG = "<your-application-aud-tag>"
```

## Local Development

1. **Install Dependencies:**
Ensure Rust, the `wasm32-unknown-unknown` target, and `cloudflared` are installed.

2. **Authenticate with Cloudflare Access:**
```bash
cloudflared access login https://edge-zero-trust-proxy.<your-subdomain>.workers.dev
```


3. **Start the Local Proxy and Mock Backend If Needed:**

Run the following command in ./edge-zero-trust-proxy/ for local proxy, run in ./mock-backend for mock backend server
```bash
npx wrangler dev
```


4. **Generate a Session Token for Testing:**
```bash
cloudflared access token https://edge-zero-trust-proxy.<your-subdomain>.workers.dev
```

Attach the resulting token to the `Cf-Access-Jwt-Assertion` header in your HTTP client (e.g., Postman).

## Future Considerations

* **Edge KV Caching for AI Token Reduction:** Implement SHA-256 payload hashing combined with Cloudflare Workers KV to cache security classifications for recurring idempotent payloads, minimizing model invocation latency and API cost.
* **Service Bindings:** Transition upstream routing from public subrequests to native Cloudflare Service Bindings for zero-latency, private intra-worker communication.
* **Granular Role-Based Access Control (RBAC):** Extend JWT claim decoding to enforce route-level authorization based on user groups and identity metadata.