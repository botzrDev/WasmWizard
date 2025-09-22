# Wasm Wizard Development Setup

## Quick Start

1. **Start the development database:**
   ```bash
   docker-compose -f docker-compose.dev.yml up -d
   ```

2. **Copy environment variables:**
   ```bash
   cp .env.development .env
   ```

3. **Run the application:**
   ```bash
   cargo run
   ```

4. **Open your browser:**
   - Frontend: http://localhost:8080
   - Health check: http://localhost:8080/health
   - Metrics: http://localhost:8080/metrics

## Configuration

The application uses environment-based configuration:

- **Development**: Auth disabled, local PostgreSQL, debug logging
- **Staging**: Auth enabled, external database required
- **Production**: Auth enabled, external database required, optimized settings

### Environment Variables

| Variable | Development Default | Description |
|----------|-------------------|-------------|
| `DATABASE_URL` | `postgres://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev` | Database connection |
| `AUTH_REQUIRED` | `false` | Enable/disable authentication |
| `ENVIRONMENT` | `development` | Runtime environment |
| `LOG_LEVEL` | `debug` | Logging verbosity |

## Database Management

**Start database:**
```bash
docker-compose -f docker-compose.dev.yml up -d db redis
```

**View database (optional):**
```bash
docker-compose -f docker-compose.dev.yml --profile tools up -d pgadmin
# Visit http://localhost:5050 (admin@wasm-wizard.dev / admin)
```

**Reset database:**
```bash
docker-compose -f docker-compose.dev.yml down -v
docker-compose -f docker-compose.dev.yml up -d
```

## Development Features

- ✅ **No authentication required** - Test immediately
- ✅ **Auto database migrations** - Setup handled automatically
- ✅ **Hot reload ready** - Use `cargo watch -x run` for live reload
- ✅ **Debug logging** - Detailed request/response logging
- ✅ **Health checks** - Monitor application status

## End-to-End Testing with Cypress

Run the Cypress smoke tests after the API is up and reachable at `http://localhost:8080`.

1. **Install Node dependencies (from the repository root):**
   ```bash
   # Skip downloading the Cypress binary during dependency install in restricted environments
   CYPRESS_INSTALL_BINARY=0 npm ci
   ```
   - Requires Node.js 18 or newer and npm 9+.
2. **Start the WasmWizard application:** follow the steps in [Quick Start](#quick-start) above.
3. **Execute the headless Cypress suite:**
   ```bash
   npm run cypress:run
   ```

To debug interactively, use `npm run cypress:open`, which launches the Cypress runner pointed at `http://localhost:8080`.

> [!TIP]
> Cypress downloads its binary the first time the test suite runs. If you are behind a proxy or
> have outbound network restrictions, run `npx cypress install --force` from an environment with
> access to `https://download.cypress.io/` and cache the resulting `~/.cache/Cypress` directory for
> reuse in CI.

### Continuous Integration

CI environments can optionally execute the Cypress smoke suite using the opt-in `cypress-smoke` job
defined in [`.github/workflows/ci.yml`](../.github/workflows/ci.yml). The job leverages the
`cypress-io/github-action`’s underlying utilities (such as `wait-on`) and only runs when the
`RUN_CYPRESS_SMOKE` repository variable is set to `true`, enabling gradual rollout or gated
nightly execution.

When enabling the job:

- Ensure the WasmWizard API is reachable at `http://localhost:8080` (for GitHub Actions, this is the
  default service URL once the application is started).
- Pre-populate the Cypress cache by adding a step such as `npx cypress install` or by restoring a
  cached `~/.cache/Cypress` directory.
- Export `CYPRESS_INSTALL_BINARY=0` during `npm install` steps if you restore the cached binary to
  avoid redundant downloads.

## Architecture

This follows professional development patterns:
- **Same codebase** for all environments
- **Feature flags** (auth_required) instead of special modes
- **Environment-based configuration**
- **Docker for dependencies** 
- **PostgreSQL everywhere** (development matches production)

## Onboarding New Developers

1. Install prerequisites: Rust, Docker
2. Clone repo and run `docker-compose -f docker-compose.dev.yml up -d`
3. Copy `.env.development` to `.env`
4. Run `cargo run`
5. Visit http://localhost:8080

**No complex setup, no special configuration, no "works on my machine" issues.**
