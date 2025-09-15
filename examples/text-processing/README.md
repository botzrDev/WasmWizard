# Text Processing Example

This example demonstrates advanced text processing capabilities in WebAssembly, including:

- Word counting and frequency analysis
- Sentiment analysis (basic)
- Text format conversion
- String manipulation and memory management

## 📁 Files

- `src/lib.rs` - Text processing algorithms in Rust
- `Cargo.toml` - Project configuration
- `build.sh` - Build script
- `test.sh` - Comprehensive testing
- `benchmark.sh` - Performance analysis
- `README.md` - This documentation

## 🚀 Quick Start

```bash
# Build the WASM module
./build.sh

# Test basic functionality
./test.sh

# Run performance benchmarks
./benchmark.sh
```

## 📖 Text Processing Functions

### Word Count Analysis
```bash
curl -X POST http://localhost:8080/api/v1/execute \
  -H "Content-Type: application/json" \
  -d '{
    "wasm_module": "'"$(base64 -w 0 text_processing.wasm)"'",
    "function": "analyze_text",
    "args": ["Hello world! This is a test sentence."]
  }'
```

### Sentiment Analysis
```bash
curl -X POST http://localhost:8080/api/v1/execute \
  -H "Content-Type: application/json" \
  -d '{
    "wasm_module": "'"$(base64 -w 0 text_processing.wasm)"'",
    "function": "sentiment_score",
    "args": ["I love this amazing product! It works perfectly."]
  }'
```

## 🔧 Key Features

- **Memory Management**: Efficient handling of dynamic strings
- **Unicode Support**: Proper UTF-8 encoding/decoding
- **Performance Optimized**: Fast algorithms for text processing
- **Error Handling**: Robust error handling for malformed input

## 📊 Performance Characteristics

- **Word Count**: O(n) time complexity
- **Memory Usage**: Scales with input size
- **Unicode Handling**: Full UTF-8 support
- **Batch Processing**: Efficient for multiple texts

## 🎯 Use Cases

- Content analysis and moderation
- Text mining and NLP preprocessing
- Format conversion and normalization
- Real-time text processing pipelines

## 📚 Related Examples

- **[Hello World](../hello-world/)** - Basic concepts
- **[Fibonacci](../fibonacci/)** - Performance optimization
- **[API Integration](../api-integration/)** - Production usage