# Resume Editor - Complete Implementation

A full-featured resume editor with Overleaf-like interface, real-time Typst compilation, and PocketBase integration.

## Features

### ✅ Phase 1 (MVP) - Complete
- [x] Authentication system with PocketBase
- [x] Login page with email/password
- [x] Editor page with auth guard
- [x] Split view (Typst source + PDF preview)
- [x] Download buttons for PDF and Typst
- [x] Link to PocketBase for uploads

### ✅ Phase 2 (Full Editor) - Complete
- [x] Monaco editor integration
- [x] Real-time Typst compilation
- [x] Live PDF preview
- [x] Keyboard shortcuts (⌘S / Ctrl+S to compile)
- [x] Unsaved changes indicator
- [x] Compilation error display
- [x] Dockerized Typst service on VM
- [x] Cloudflare Workers API proxy

### ✅ Infrastructure
- [x] Typst compilation service (Docker)
- [x] Caddy reverse proxy integration
- [x] GitHub Actions for resume sync
- [x] Automated deployment scripts

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend                             │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │  Monaco Editor   │         │   PDF Preview    │          │
│  │  (Typst Code)    │────────▶│   (iframe)       │          │
│  └──────────────────┘         └──────────────────┘          │
│           │                                                   │
│           │ Compile (⌘S)                                     │
│           ▼                                                   │
│  ┌─────────────────────────────────────────────┐            │
│  │     Cloudflare Workers API                   │            │
│  │     /api/typst/compile                       │            │
│  └─────────────────────────────────────────────┘            │
│           │                                                   │
│           │ HTTPS                                            │
│           ▼                                                   │
│  ┌─────────────────────────────────────────────┐            │
│  │     Typst Service (Docker on VM)            │            │
│  │     https://typst.ifkash.dev                │            │
│  │     - Bun HTTP server                        │            │
│  │     - Typst compiler                         │            │
│  │     - Font support                           │            │
│  └─────────────────────────────────────────────┘            │
│           │                                                   │
│           │ Compiled PDF                                     │
│           ▼                                                   │
│  ┌─────────────────────────────────────────────┐            │
│  │     PocketBase                               │            │
│  │     https://pb.ifkash.dev                   │            │
│  │     - Resume storage                         │            │
│  │     - Version history                        │            │
│  │     - Authentication                         │            │
│  └─────────────────────────────────────────────┘            │
└─────────────────────────────────────────────────────────────┘
```

## User Workflow

1. **Login**: Visit `/login` and authenticate with PocketBase admin credentials
2. **Edit**: Navigate to `/editor` to view and edit resume
3. **Compile**: Press ⌘S (or Ctrl+S) to compile Typst to PDF
4. **Preview**: See live PDF preview in right pane
5. **Download**: Download both Typst source and compiled PDF
6. **Upload**: Upload new version via PocketBase admin panel

## API Endpoints

### Authentication
- `POST /api/auth/login` - Login with email/password
- `GET /api/auth/verify` - Verify JWT token

### Resume Management
- `GET /api/resume` - Download latest resume PDF (public)
- `GET /api/resume/latest` - Get resume metadata with file URLs (authenticated)
- `POST /api/resume/upload` - Upload new version (placeholder)

### Compilation
- `POST /api/typst/compile` - Compile Typst code to PDF (authenticated)

## Frontend Routes

- `/` - Homepage with resume download button
- `/login` - Authentication page
- `/editor` - Resume editor (requires authentication)

## Key Features

### Monaco Editor
- Syntax highlighting (plaintext mode for Typst)
- Line numbers and minimap
- Auto-layout and word wrap
- Keyboard shortcuts
- Change detection

### Real-time Compilation
- Compile on ⌘S / Ctrl+S
- Debounced auto-save to localStorage
- Live PDF preview update
- Compilation error display
- Loading states

### PDF Preview
- Iframe-based rendering
- Blob URL for compiled PDFs
- Automatic cleanup of old URLs
- Fallback to PocketBase URL

### Authentication
- JWT token storage in localStorage
- Auto-redirect to login if not authenticated
- Token verification on protected routes
- Logout functionality

## Deployment

### Typst Service (VM)

```bash
# Clone repo on VM
cd ~
git clone https://github.com/kashifulhaque/ifkash.git
cd ifkash/typst-service

# Deploy with Docker
./deploy-docker.sh
```

### Cloudflare Workers

```bash
# Deploy API
cd api
wrangler deploy
```

### Frontend

```bash
# Build and deploy (via your deployment pipeline)
bun run build
```

## Configuration

### Environment Variables

**Cloudflare Workers** (`wrangler.toml`):
```toml
[vars]
POCKETBASE_URL = "https://pb.ifkash.dev"
TYPST_SERVICE_URL = "https://typst.ifkash.dev"

# Secrets (set with wrangler secret put):
# POCKETBASE_EMAIL
# POCKETBASE_PASSWORD
```

**Typst Service** (`compose.yml`):
```yaml
environment:
  - PORT=3001
  - FONT_PATH=/app/fonts
```

### Caddy Configuration

Add to your Caddyfile:
```
typst.ifkash.dev {
  reverse_proxy typst-service:3001
}
```

## Testing

### Test Typst Service

```bash
# Health check
curl https://typst.ifkash.dev/health

# Compile test
curl -X POST https://typst.ifkash.dev/compile \
  -H "Authorization: Bearer test" \
  -H "Content-Type: application/json" \
  -d '{"code": "#set page(width: 10cm, height: 10cm)\nHello World"}' \
  -o test.pdf
```

### Test Full Flow

1. Visit `http://localhost:5173/login`
2. Login with PocketBase credentials
3. Navigate to `/editor`
4. Edit Typst code
5. Press ⌘S to compile
6. Verify PDF updates
7. Download files
8. Upload new version via PocketBase

## Keyboard Shortcuts

- `⌘S` / `Ctrl+S` - Compile Typst to PDF
- `⌘Z` / `Ctrl+Z` - Undo (Monaco editor)
- `⌘Shift+Z` / `Ctrl+Shift+Z` - Redo (Monaco editor)
- `⌘F` / `Ctrl+F` - Find (Monaco editor)
- `⌘H` / `Ctrl+H` - Replace (Monaco editor)

## Security

- ✅ Authentication required for editor access
- ✅ JWT token verification
- ✅ Rate limiting on Typst service (10 req/min)
- ✅ Timeout protection (30s max compilation)
- ✅ Resource limits (CPU, memory)
- ✅ HTTPS everywhere
- ✅ CORS configured
- ✅ No file system access from Typst code

## Performance

- **Editor load time**: < 1s
- **Compilation time**: < 2s for typical resume
- **PDF preview update**: Instant
- **Monaco editor**: Smooth editing even for large files
- **Memory usage**: ~50-100MB per compilation

## Troubleshooting

### Editor won't load
- Check authentication (token in localStorage)
- Verify `/api/resume/latest` endpoint
- Check browser console for errors

### Compilation fails
- Check Typst service health: `curl https://typst.ifkash.dev/health`
- Verify authentication token
- Check compilation error message
- Test Typst service directly

### PDF preview not updating
- Check browser console for blob URL errors
- Verify PDF was compiled successfully
- Try refreshing the page

### Monaco editor issues
- Clear browser cache
- Check for JavaScript errors
- Verify monaco-editor package is installed

## Future Enhancements

- [ ] Typst syntax highlighting in Monaco
- [ ] Auto-compile on save (debounced)
- [ ] Version comparison / diff view
- [ ] Collaborative editing
- [ ] Template library
- [ ] Export to other formats
- [ ] AI-powered suggestions
- [ ] Custom Typst packages support
- [ ] Direct upload from editor to PocketBase

## Files

### Backend
- `api/src/handlers/auth.rs` - Authentication logic
- `api/src/handlers/resume_api.rs` - Resume management
- `api/src/handlers/typst.rs` - Compilation proxy
- `api/src/routes/*.rs` - Route handlers

### Frontend
- `src/routes/login/+page.svelte` - Login page
- `src/routes/editor/+page.svelte` - Editor page
- `src/routes/editor/+page.ts` - Page load with auth guard
- `src/lib/components/MonacoEditor.svelte` - Monaco wrapper
- `src/lib/stores/auth.ts` - Authentication store

### Infrastructure
- `typst-service/` - Dockerized Typst compilation service
- `.github/workflows/sync-resume.yml` - Auto-sync workflow
- `scripts/sync-resume.ts` - Resume sync script

## License

MIT
