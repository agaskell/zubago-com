# Zubago marketing site

# Start local dev server for the static site
serve port="8765":
    python3 -m http.server {{port}}

# Deploy the static site to Cloudflare Pages
deploy:
    #!/usr/bin/env bash
    tmp=$(mktemp -d)
    cp index.html style.css main.js "$tmp/"
    npx wrangler pages deploy "$tmp" --project-name=zubago --commit-message="Deploy $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    rm -rf "$tmp"

# Start the worker dev server
worker-dev:
    cd worker && npx wrangler dev

# Clean worker build artifacts
worker-clean:
    rm -rf worker/build worker/target

# Deploy the worker to Cloudflare
worker-deploy:
    cd worker && npx wrangler deploy

# Create the D1 database (run once)
db-create:
    cd worker && npx wrangler d1 create zubago-contact

# Run D1 migrations locally
db-migrate-local:
    cd worker && npx wrangler d1 migrations apply zubago-contact --local

# Run D1 migrations on production
db-migrate:
    cd worker && npx wrangler d1 migrations apply zubago-contact --remote

# Set the Turnstile secret key
set-turnstile-secret:
    cd worker && npx wrangler secret put TURNSTILE_SECRET_KEY
