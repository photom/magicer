# magicer
Return file magic. Rust REST API Server. Just for etude.

## API Documentation

The OpenAPI specification is located at `api/v1/openapi.yaml`.

### Automated Documentation Generation

HTML documentation is automatically generated from the OpenAPI spec via GitHub Actions:

- **Trigger**: Runs on every push to `main` branch when `openapi.yaml` changes
- **Output**: Static HTML hosted on GitHub Pages
- **Validation**: Lints the OpenAPI spec before generating docs
- **Access**: Visit the GitHub Pages URL after deployment completes

To manually trigger the workflow:
1. Go to Actions tab in GitHub
2. Select "Generate OpenAPI Documentation"
3. Click "Run workflow"

### Local Documentation Generation

To generate documentation locally:

```bash
# Install Redocly CLI
npm install -g @redocly/cli

# Validate the spec
redocly lint api/v1/openapi.yaml

# Generate HTML
redocly build-docs api/v1/openapi.yaml --output docs.html
```

## References
