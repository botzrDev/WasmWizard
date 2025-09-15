# Wasm Wizard Examples

Welcome to the Wasm Wizard examples! This directory contains practical examples that demonstrate how to use Wasm Wizard for various use cases. Each example includes:

- 📁 **Source Code**: Rust source code for the WASM module
- 🔧 **Build Scripts**: Scripts to compile the WASM module
- 📡 **API Examples**: curl commands and code samples showing how to interact with the API
- 📖 **Tutorials**: Step-by-step guides explaining the concepts
- 📊 **Benchmarks**: Performance metrics and comparisons

## 🚀 Quick Start

1. **Start Wasm Wizard** (if not already running):
   ```bash
   cd wasmwizard
   docker-compose -f docker-compose.dev.yml up -d
   cargo run
   ```

2. **Choose an Example**: Pick an example that matches your use case

3. **Follow the Tutorial**: Each example has a detailed README with step-by-step instructions

4. **Run the Example**: Use the provided scripts and API calls to see it in action

## 📚 Available Examples

### 🌟 [Hello World](./hello-world/)
**Difficulty**: Beginner
**Concepts**: Basic WASM execution, simple functions
**Use Case**: First steps with Wasm Wizard

A simple "Hello World" example that demonstrates the basic workflow of compiling a Rust function to WASM and executing it through the Wasm Wizard API.

### 🔢 [Fibonacci](./fibonacci/)
**Difficulty**: Beginner
**Concepts**: Mathematical computations, performance optimization
**Use Case**: CPU-intensive calculations

Calculate Fibonacci numbers using recursive and iterative algorithms, demonstrating performance differences and optimization techniques.

### 📝 [Text Processing](./text-processing/)
**Difficulty**: Intermediate
**Concepts**: String manipulation, memory management, complex data structures
**Use Case**: Text analysis and transformation

Process text data including word counting, sentiment analysis, and format conversion, showcasing WASM's capabilities for data processing tasks.

### 🔗 [API Integration](./api-integration/)
**Difficulty**: Advanced
**Concepts**: Authentication, complex workflows, error handling, real-world integration
**Use Case**: Production API integration

A complete example showing how to integrate Wasm Wizard into a production application with authentication, error handling, and monitoring.

## 🏗️ Building Examples

Each example can be built independently:

```bash
# Navigate to the example directory
cd examples/hello-world

# Build the WASM module
./build.sh

# Or build manually
cd src
cargo build --target wasm32-wasi --release
```

## 🧪 Testing Examples

Most examples include test scripts:

```bash
# Run the example test
./test.sh

# Or test manually with curl
curl -X POST http://localhost:8080/api/v1/execute \
  -H "Content-Type: application/json" \
  -d @test-payload.json
```

## 📊 Performance Benchmarks

Each example includes performance benchmarks comparing:
- **Native Rust execution** vs **WASM execution**
- **Different optimization levels**
- **Memory usage patterns**
- **Execution time variations**

Run benchmarks with:
```bash
./benchmark.sh
```

## 🔧 Prerequisites

Before running examples, ensure you have:

- **Rust 1.81+** with `wasm32-wasi` target:
  ```bash
  rustup target add wasm32-wasi
  ```

- **Wasm Wizard running** on `http://localhost:8080`

- **curl** or **Postman** for API testing

- **Docker** (for some examples)

## 📖 Learning Path

We recommend following this learning path:

1. **Start with Hello World** - Understand the basic concepts
2. **Try Fibonacci** - Learn about performance optimization
3. **Explore Text Processing** - Work with complex data structures
4. **Master API Integration** - Build production-ready applications

## 🤝 Contributing Examples

Want to add your own example? See our [Contributing Guide](../CONTRIBUTING.md) for guidelines on:

- Example structure and naming conventions
- Documentation requirements
- Testing and benchmarking standards
- Code quality expectations

## 📞 Support

- **Documentation**: [Wasm Wizard Docs](https://wasmwizard.dev/docs)
- **Issues**: [GitHub Issues](https://github.com/botzrDev/WasmWizard/issues)
- **Discussions**: [GitHub Discussions](https://github.com/botzrDev/WasmWizard/discussions)

## 📈 Example Metrics

| Example | Lines of Code | WASM Size | Avg Execution Time | Memory Usage |
|---------|---------------|-----------|-------------------|--------------|
| Hello World | 25 | 1.2 KB | 0.5ms | 8 KB |
| Fibonacci | 45 | 2.1 KB | 15ms | 12 KB |
| Text Processing | 120 | 8.5 KB | 25ms | 64 KB |
| API Integration | 200 | 15.2 KB | 50ms | 128 KB |

*Metrics are approximate and depend on input size and optimization level*

---

Happy coding with Wasm Wizard! 🚀