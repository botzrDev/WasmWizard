# WasmWiz API Documentation

## Overview

The WasmWiz API provides a secure, scalable platform for executing WebAssembly modules with comprehensive monitoring, authentication, and rate limiting.

**Base URL:** `https://your-domain.com`  
**Authentication:** API Key (Bearer token)  
**Content Type:** `application/json` (except where noted)

## Authentication

All API requests require authentication using an API key in the `Authorization` header:

```
Authorization: Bearer wasmwiz_your_api_key_here
```

### Getting an API Key

API keys can be generated through the web interface or programmatically:

**Web Interface:** Visit `/api-keys` in your browser  
**Rate Limits:** API key generation is rate-limited to prevent abuse

## Endpoints

### Health & Monitoring

#### GET /health

Comprehensive health check endpoint that verifies all system components.

**Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "checks": {
    "database": {
      "status": "healthy",
      "message": "Connected"
    },
    "filesystem": {
      "status": "healthy",
      "message": "Writable"
    },
    "system": {
      "status": "healthy",
      "message": "CPU: 25%, Memory: 40%"
    }
  }
}
```

**Response (503 Service Unavailable):**
```json
{
  "status": "unhealthy",
  "timestamp": "2024-01-01T12:00:00Z",
  "checks": {
    "database": {
      "status": "unhealthy",
      "message": "Connection timeout"
    }
  }
}
```

#### GET /health/live

Kubernetes liveness probe - quick check for application responsiveness.

**Response (200 OK):**
```json
{
  "status": "ok",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

#### GET /health/ready

Kubernetes readiness probe - comprehensive check including dependencies.

**Response (200 OK):**
```json
{
  "status": "ready",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

#### GET /metrics

Prometheus metrics endpoint for monitoring and alerting.

**Response (200 OK):**
```
# HELP wasmwiz_requests_total Total number of requests
# TYPE wasmwiz_requests_total counter
wasmwiz_requests_total{method="POST",endpoint="/api/wasm/execute",status="200"} 12543

# HELP wasmwiz_execution_duration_seconds WASM execution duration
# TYPE wasmwiz_execution_duration_seconds histogram
wasmwiz_execution_duration_seconds_bucket{le="0.1"} 1234
```

### WebAssembly Operations

#### POST /api/wasm/execute

Execute a WebAssembly module with provided input data.

**Content-Type:** `multipart/form-data`  
**Rate Limited:** Yes (varies by subscription tier)

**Request Body:**
- `wasm` (file, required): WebAssembly module binary
- `input` (text, optional): UTF-8 encoded input data

**Example Request (curl):**
```bash
curl -X POST \
  -H "Authorization: Bearer wasmwiz_your_key" \
  -F "wasm=@hello.wasm" \
  -F "input=Hello World" \
  https://api.wasmwiz.com/api/wasm/execute
```

**Response (200 OK - Success):**
```json
{
  "output": "Hello World processed by WASM module",
  "error": null
}
```

**Response (422 Unprocessable Entity - Execution Error):**
```json
{
  "output": null,
  "error": "WASM execution timeout"
}
```

**Error Responses:**
- `400 Bad Request`: Invalid WASM format or input data
- `401 Unauthorized`: Missing or invalid API key
- `413 Payload Too Large`: WASM or input exceeds size limits
- `429 Too Many Requests`: Rate limit exceeded

**Limits:**
- WASM module: 10MB max (configurable)
- Input data: 1MB max (configurable)
- Execution timeout: 5 seconds (configurable)
- Memory limit: 128MB (configurable)

### Authentication & API Keys

#### POST /api/auth/keys

Generate a new API key for the authenticated user.

**Response (201 Created):**
```json
{
  "api_key": "wasmwiz_abc123def456789...",
  "api_key_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Security Note:** The `api_key` field contains the plain text key that will be shown only once. Store it securely as it cannot be retrieved again.

#### GET /api/auth/keys

List all API keys for the authenticated user.

**Response (200 OK):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "tier": "basic",
    "is_active": true,
    "created_at": "2024-01-01T10:00:00Z",
    "last_used": "2024-01-01T12:00:00Z"
  }
]
```

#### DELETE /api/auth/keys/{id}

Revoke an API key.

**Response (204 No Content):** Key successfully revoked

**Error Responses:**
- `404 Not Found`: API key not found or doesn't belong to user
- `401 Unauthorized`: Invalid authentication

### Web Interface

#### GET /

Main application dashboard with WASM execution interface.

**Response:** HTML page with upload form and user interface

#### GET /api-keys

API key management interface.

**Response:** HTML page for viewing and managing API keys

## Rate Limiting

Rate limits vary by subscription tier:

| Tier | Requests/Minute | Requests/Hour | Burst Limit |
|------|----------------|---------------|-------------|
| Free | 10 | 100 | 5 |
| Basic | 100 | 1000 | 20 |
| Pro | 1000 | 10000 | 100 |
| Enterprise | Unlimited | Unlimited | Unlimited |

**Rate Limit Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
Retry-After: 60 (when exceeded)
```

## Error Handling

All API errors follow a consistent format:

```json
{
  "error": "Human-readable error message"
}
```

**Common HTTP Status Codes:**
- `200 OK`: Success
- `201 Created`: Resource created
- `204 No Content`: Success with no content
- `400 Bad Request`: Invalid request
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `413 Payload Too Large`: Request too large
- `422 Unprocessable Entity`: Valid request but cannot process
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

## SDKs and Client Libraries

### JavaScript/TypeScript

```javascript
class WasmWizClient {
  constructor(apiKey) {
    this.apiKey = apiKey;
    this.baseUrl = 'https://api.wasmwiz.com';
  }

  async executeWasm(wasmFile, input = '') {
    const formData = new FormData();
    formData.append('wasm', wasmFile);
    formData.append('input', input);

    const response = await fetch(`${this.baseUrl}/api/wasm/execute`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiKey}`
      },
      body: formData
    });

    return response.json();
  }
}
```

### Python

```python
import requests

class WasmWizClient:
    def __init__(self, api_key):
        self.api_key = api_key
        self.base_url = 'https://api.wasmwiz.com'

    def execute_wasm(self, wasm_file_path, input_data=''):
        with open(wasm_file_path, 'rb') as f:
            files = {'wasm': f}
            data = {'input': input_data}

            response = requests.post(
                f'{self.base_url}/api/wasm/execute',
                files=files,
                data=data,
                headers={'Authorization': f'Bearer {self.api_key}'}
            )

        return response.json()
```

## Best Practices

### Error Handling
```javascript
try {
  const result = await client.executeWasm(wasmFile, input);
  if (result.error) {
    console.error('WASM execution failed:', result.error);
  } else {
    console.log('Output:', result.output);
  }
} catch (error) {
  if (error.status === 429) {
    // Rate limited - implement backoff
    await delay(error.headers['Retry-After'] * 1000);
  } else {
    console.error('Request failed:', error);
  }
}
```

### File Upload
- Validate WASM files before upload
- Compress large modules if possible
- Handle network timeouts gracefully
- Implement progress indicators for large files

### Rate Limiting
- Monitor `X-RateLimit-Remaining` header
- Implement exponential backoff on 429 errors
- Cache results when appropriate
- Consider upgrading subscription tier for higher limits

## Support

- **Documentation:** https://docs.wasmwiz.com
- **API Status:** https://status.wasmwiz.com
- **Support:** support@wasmwiz.com
- **Community:** https://community.wasmwiz.com