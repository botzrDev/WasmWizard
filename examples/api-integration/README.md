# API Integration Example

This comprehensive example demonstrates production-ready integration with Wasm Wizard, including:

- **Authentication**: JWT token handling and API key management
- **Error Handling**: Robust error handling and recovery
- **Monitoring**: Performance tracking and logging
- **Batch Processing**: Efficient handling of multiple requests
- **Configuration**: Environment-based configuration management

## 📁 Files

- `src/lib.rs` - WASM module with integration functions
- `Cargo.toml` - Project configuration
- `build.sh` - Build script
- `test.sh` - Integration testing
- `benchmark.sh` - Performance analysis
- `integration_test.py` - Python integration test
- `README.md` - This documentation

## 🚀 Quick Start

### 1. Build the WASM Module
```bash
./build.sh
```

### 2. Set Up Authentication
```bash
# Get API key from Wasm Wizard
export WASM_WIZARD_API_KEY="your-api-key-here"
export WASM_WIZARD_URL="http://localhost:8080"
```

### 3. Run Integration Tests
```bash
# Python integration test
python3 integration_test.py

# Or manual testing
./test.sh
```

## 🔐 Authentication

### API Key Setup
```bash
# Register for an API key
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "tier": "developer"
  }'
```

### Using API Keys
```bash
curl -X POST http://localhost:8080/api/v1/execute \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "wasm_module": "'"$(base64 -w 0 api_integration.wasm)"'",
    "function": "process_data",
    "args": ["input data"]
  }'
```

## 📊 Monitoring and Logging

### Performance Tracking
```bash
curl -X GET http://localhost:8080/api/v1/metrics \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### Request Logging
All requests are automatically logged with:
- Execution time
- Memory usage
- Error details
- Request metadata

## 🧪 Integration Testing

### Python Integration Test
```python
import requests
import base64

# Load WASM module
with open('api_integration.wasm', 'rb') as f:
    wasm_data = base64.b64encode(f.read()).decode()

# API request
response = requests.post(
    'http://localhost:8080/api/v1/execute',
    headers={
        'Authorization': f'Bearer {API_KEY}',
        'Content-Type': 'application/json'
    },
    json={
        'wasm_module': wasm_data,
        'function': 'batch_process',
        'args': [['data1', 'data2', 'data3']]
    }
)

print(f"Status: {response.status_code}")
print(f"Result: {response.json()}")
```

## 🔧 Error Handling

### Common Error Scenarios
- **Invalid WASM**: Malformed or corrupted modules
- **Function Not Found**: Requested function doesn't exist
- **Memory Limits**: Exceeding memory constraints
- **Timeout**: Execution taking too long
- **Authentication**: Invalid or expired API keys

### Error Recovery
```python
def execute_with_retry(wasm_data, function_name, args, max_retries=3):
    for attempt in range(max_retries):
        try:
            response = requests.post(
                f"{API_URL}/api/v1/execute",
                headers=get_auth_headers(),
                json={
                    'wasm_module': wasm_data,
                    'function': function_name,
                    'args': args
                },
                timeout=30
            )

            if response.status_code == 200:
                return response.json()
            elif response.status_code == 429:  # Rate limited
                time.sleep(2 ** attempt)  # Exponential backoff
                continue
            else:
                raise Exception(f"API Error: {response.status_code}")

        except requests.exceptions.Timeout:
            if attempt == max_retries - 1:
                raise Exception("Request timeout")
            continue

    raise Exception("Max retries exceeded")
```

## 📈 Performance Optimization

### Batch Processing
```bash
# Process multiple items efficiently
curl -X POST http://localhost:8080/api/v1/execute \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "wasm_module": "'"$(base64 -w 0 api_integration.wasm)"'",
    "function": "batch_process",
    "args": [["item1", "item2", "item3", "item4", "item5"]]
  }'
```

### Connection Pooling
```python
# Reuse connections for better performance
session = requests.Session()
session.headers.update(get_auth_headers())

# Reuse session for multiple requests
for data in dataset:
    response = session.post(
        f"{API_URL}/api/v1/execute",
        json={
            'wasm_module': wasm_data,
            'function': 'process_item',
            'args': [data]
        }
    )
```

## 🔒 Security Best Practices

### API Key Management
- Rotate keys regularly
- Use different keys for different environments
- Store keys securely (environment variables, not code)
- Monitor key usage patterns

### Input Validation
- Validate all inputs before sending to WASM
- Sanitize data to prevent injection attacks
- Check file sizes and types for uploads
- Implement rate limiting on client side

### Error Handling Security
- Don't expose internal error details to users
- Log security events appropriately
- Implement proper access controls
- Use HTTPS for all API communications

## 📊 Production Deployment

### Environment Configuration
```bash
# Production settings
export WASM_WIZARD_URL="https://api.wasmwizard.dev"
export WASM_WIZARD_API_KEY="prod-api-key"
export REQUEST_TIMEOUT=30
export MAX_RETRIES=3
export BATCH_SIZE=100
```

### Health Monitoring
```bash
# Health check
curl -f https://api.wasmwizard.dev/health

# Metrics endpoint
curl -H "Authorization: Bearer ${API_KEY}" \
     https://api.wasmwizard.dev/metrics
```

## 🎯 Use Cases

- **Data Processing Pipelines**: ETL operations with WASM
- **Real-time Analytics**: Streaming data processing
- **API Gateways**: Custom logic execution
- **Microservices**: Lightweight function execution
- **Edge Computing**: Distributed processing

## 📚 Advanced Features

- **Streaming**: Process large datasets in chunks
- **Caching**: Intelligent result caching
- **Async Processing**: Background job processing
- **Webhooks**: Event-driven processing
- **Multi-tenancy**: Isolated execution environments

This example provides a complete foundation for production WASM integration! 🚀