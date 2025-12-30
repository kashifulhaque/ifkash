# PocketBase Setup Guide for Resume API

This guide will help you set up your PocketBase instance to work with the `/api/resume` endpoint.

## Step 1: Configure Cloudflare Worker Secrets

Set your PocketBase admin credentials as Cloudflare Worker secrets:

```bash
cd /Users/kashif/Documents/projects/ifkash/api

# Set your PocketBase admin email
wrangler secret put POCKETBASE_EMAIL
# Enter: haque.kashiful7@gmail.com

# Set your PocketBase admin password
wrangler secret put POCKETBASE_PASSWORD
# Enter: your-pocketbase-password
```

> [!NOTE]
> The API will authenticate with PocketBase on each request using these credentials to obtain a JWT token.

## Step 2: Create the `resumes` Collection

1. In your PocketBase admin panel, go to **Collections**
2. Click **New Collection**
3. Choose **Base collection**
4. Set the collection name to: `resumes`

### Add Fields

Add the following fields to the collection:

#### Field 1: `pdf_file` (File)
- **Type**: File
- **Max select**: 1
- **Max size**: 10485760 (10MB)
- **MIME types**: `application/pdf`
- **Required**: Yes

#### Field 2: `typst_file` (File)
- **Type**: File
- **Max select**: 1
- **Max size**: 5242880 (5MB)
- **MIME types**: Leave empty or add `text/plain`
- **Required**: No

#### Field 3: `version` (Text)
- **Type**: Text
- **Required**: No
- **Min/Max**: Leave default
- **Pattern**: Leave empty

#### Field 4: `notes` (Text)
- **Type**: Text
- **Required**: No
- **Min/Max**: Leave default

### Configure API Rules

In the collection settings, configure the API rules:

- **List/View Rule**: Leave empty (public access) or use `""` for fully public
- **Create Rule**: Leave empty (admin only)
- **Update Rule**: Leave empty (admin only)
- **Delete Rule**: Leave empty (admin only)

Click **Save** to create the collection.

## Step 3: Upload Your First Resume

1. Go to the `resumes` collection in the admin panel
2. Click **New Record**
3. Upload your PDF file in the `pdf_file` field
4. Upload your Typst source file in the `typst_file` field (optional)
5. Add a version identifier (e.g., "2024-12" or "v1.0") in the `version` field (optional)
6. Add any notes about this version in the `notes` field (optional)
7. Click **Create**

## Step 4: Test the Endpoint

### Build and Deploy

```bash
cd /Users/kashif/Documents/projects/ifkash/api

# Build the worker
cargo build --target wasm32-unknown-unknown --release

# Test locally
wrangler dev
```

### Test Requests

In another terminal:

```bash
# Download the PDF (default behavior)
curl http://localhost:8787/api/resume -o test_resume.pdf

# Get JSON metadata with download URL
curl "http://localhost:8787/api/resume?format=json"

# Get just the PocketBase URL
curl "http://localhost:8787/api/resume?format=url"
```

### Verify

- [ ] PDF downloads successfully
- [ ] Downloaded file is named `Kashiful_Haque.pdf`
- [ ] `?format=json` returns metadata including version, dates, and download URL
- [ ] `?format=url` returns just the PocketBase file URL

## Step 5: Deploy to Production

Once testing is successful:

```bash
wrangler deploy
```

Your endpoint will be available at:
- `https://ifkash.dev/api/resume`
- `https://ifkash-api.workers.dev/api/resume` (workers.dev preview URL)

## Managing Resume Versions

To upload a new version of your resume:

1. Go to the `resumes` collection in PocketBase admin
2. Click **New Record** (don't edit the existing one)
3. Upload the new PDF and Typst files
4. Update the version number
5. Add notes about what changed

The API will automatically serve the latest version (sorted by creation date).

## Troubleshooting

### "No resume found in PocketBase"
- Ensure you've created at least one record in the `resumes` collection
- Check that the collection name is exactly `resumes` (lowercase, plural)

### "PocketBase authentication failed (401)"
- Verify the email and password secrets are set correctly: `wrangler secret list`
- Check that the credentials match your PocketBase admin account
- Try logging in to the PocketBase admin panel with the same credentials

### "Failed to download PDF from PocketBase"
- Check that the PDF file was uploaded successfully
- Verify the file isn't corrupted
- Ensure the file size is within limits

### CORS Issues
- The API already has CORS headers configured in `lib.rs`
- All endpoints allow cross-origin requests from any domain
