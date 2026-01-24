# OSS Explorer

A web application to discover and explore open source software on GitHub. Filter by programming language, sort by stars, forks, issues, or creation date.

Built with Rust + Leptos (WebAssembly).

## Features

- Search GitHub repositories by keyword
- Filter by 25+ programming languages
- Sort by:
  - Stars
  - Forks
  - Open issues
  - Creation date
  - Last updated
- Ascending/descending order
- Direct links to GitHub repos
- Responsive design with dark theme

## Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Trunk](https://trunkrs.dev/) - WASM build tool
- wasm32-unknown-unknown target

## Installation

```bash
# Install trunk
cargo install trunk

# Add WASM target
rustup target add wasm32-unknown-unknown
```

## Development

```bash
# Run development server with hot reload
trunk serve

# Open http://localhost:8080
```

## Production Build

```bash
# Build optimized WASM bundle
trunk build --release

# Output in ./dist/
```

## Deployment

The `dist/` folder contains static files that can be deployed to any static hosting:

- GitHub Pages
- Netlify
- Vercel
- Cloudflare Pages
- Any web server

## GitHub API Rate Limits

- Unauthenticated: 10 requests/minute for search API
- Authenticated: 30 requests/minute for search API

For higher limits, you can add a GitHub token to the request headers.

## License

MIT
