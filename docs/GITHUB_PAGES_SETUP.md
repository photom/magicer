# GitHub Pages Setup for API Documentation <!-- omit in toc -->

This document explains how to enable GitHub Pages to host the automatically generated OpenAPI documentation.
- [Prerequisites](#prerequisites)
- [Setup Steps](#setup-steps)
  - [1. Enable GitHub Pages](#1-enable-github-pages)
  - [2. Run the Workflow](#2-run-the-workflow)
  - [3. Access Your Documentation](#3-access-your-documentation)
- [Workflow Details](#workflow-details)
  - [generate-docs](#generate-docs)
  - [deploy](#deploy)
- [Troubleshooting](#troubleshooting)
  - [Workflow fails with "Permission denied"](#workflow-fails-with-permission-denied)
  - [Pages deployment fails](#pages-deployment-fails)
  - [Documentation not updating](#documentation-not-updating)
- [Artifacts](#artifacts)

## Prerequisites

- Repository must be public (or GitHub Pro/Enterprise for private repos)
- Admin access to the repository

## Setup Steps

### 1. Enable GitHub Pages

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Pages**
3. Under **Build and deployment**:
   - **Source**: Select "GitHub Actions"
4. Click **Save**

### 2. Run the Workflow

The workflow runs automatically when:
- You push changes to `main` branch that modify `api/v1/openapi.yaml`
- You manually trigger it from the Actions tab

To manually trigger:
1. Go to **Actions** tab
2. Select **Generate OpenAPI Documentation** workflow
3. Click **Run workflow** → **Run workflow**

### 3. Access Your Documentation

After the workflow completes successfully:
1. Go to **Settings** → **Pages**
2. Find the **Your site is live at** URL
3. Bookmark this URL for easy access

The URL format is typically:
```
https://<username>.github.io/<repository-name>/
```

## Workflow Details

The workflow consists of two jobs:

### generate-docs
- Validates the OpenAPI spec using Redocly CLI
- Generates static HTML documentation
- Uploads the HTML as a build artifact

### deploy
- Only runs on `main` branch pushes
- Downloads the generated HTML
- Deploys to GitHub Pages

## Troubleshooting

### Workflow fails with "Permission denied"
- Ensure the workflow has proper permissions (already configured in `openapi-docs.yml`)
- Check **Settings** → **Actions** → **General** → **Workflow permissions**
- Ensure "Read and write permissions" is enabled

### Pages deployment fails
- Verify GitHub Pages is enabled in repository settings
- Check that "Source" is set to "GitHub Actions" (not "Deploy from a branch")

### Documentation not updating
- Check the Actions tab for workflow status
- Verify the workflow completed successfully
- Clear browser cache or try incognito mode

## Artifacts

Build artifacts are retained for 90 days and can be downloaded from:
- **Actions** tab → Select a workflow run → **Artifacts** section

This is useful for:
- Previewing documentation before deploying
- Downloading documentation for offline use
- Auditing historical versions
