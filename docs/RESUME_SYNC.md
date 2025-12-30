# Resume Sync Automation

This project includes automation to keep the static resume files in `static/assets/` synchronized with the latest version in PocketBase.

## Manual Sync

To manually sync the resume files:

```bash
mise run assets:sync-resume
```

This will:
1. Authenticate with PocketBase
2. Fetch the latest resume record
3. Download the PDF and Typst files
4. Save them to `static/assets/`

## Automatic Sync

### GitHub Actions

The repository includes a GitHub Actions workflow (`.github/workflows/sync-resume.yml`) that automatically syncs resume files:

- **Daily**: Runs at midnight UTC
- **Manual**: Can be triggered from the Actions tab
- **Webhook**: Can be triggered via repository dispatch

### Setup GitHub Secrets

Add these secrets to your GitHub repository:

1. Go to Settings → Secrets and variables → Actions
2. Add the following secrets:
   - `POCKETBASE_EMAIL`: Your PocketBase admin email
   - `POCKETBASE_PASSWORD`: Your PocketBase admin password

### Triggering from PocketBase

You can set up a PocketBase hook to trigger the GitHub Action when a resume is updated:

1. In PocketBase, go to Settings → Hooks
2. Add a new hook for the `resumes` collection on `create` or `update`
3. Use this webhook URL:
   ```
   https://api.github.com/repos/kashifulhaque/ifkash/dispatches
   ```
4. Add headers:
   ```
   Authorization: Bearer YOUR_GITHUB_PAT
   Accept: application/vnd.github+json
   ```
5. Body:
   ```json
   {
     "event_type": "resume_updated"
   }
   ```

## Workflow

1. **Edit Resume**: Update your resume in PocketBase
2. **Auto-Sync**: GitHub Actions automatically syncs to static assets
3. **Deploy**: Changes are committed and pushed, triggering deployment
4. **Fallback**: Static files serve as fallback if API is down

## Files

- `scripts/sync-resume.ts` - Sync script
- `.github/workflows/sync-resume.yml` - GitHub Actions workflow
- `static/assets/Kashiful_Haque.pdf` - Static PDF (synced)
- `static/assets/*.typ` - Static Typst files (synced)
