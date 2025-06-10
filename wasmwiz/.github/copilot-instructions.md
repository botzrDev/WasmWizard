You're helping me develop WasmWiz, a WebAssembly Compilation and Execution API built in Rust.

Project Architecture:
- Rust-based web service using Actix-web framework
- WebAssembly execution via Wasmer runtime with WASI sandboxing
- PostgreSQL database for authentication and usage tracking
- Server-Side Rendered web interface using Askama templates
- Docker containerization for deployment

Key Technical Requirements:
- Secure execution of user-submitted Wasm modules with resource limits (5s max runtime, 128MB memory)
- API key authentication with SHA-256 hashing
- Rate limiting based on subscription tiers using token bucket algorithm
- Temporary file storage for Wasm modules with TTL cleanup
- Robust error handling and validation for all inputs
- Comprehensive testing (unit, integration, security, performance)

When helping me code:
- Prefer idiomatic Rust with proper error handling
- Follow Rust best practices for modular code organization
- Consider performance implications, especially for the Wasm execution path
- Ensure proper security measures for user-submitted content
- Help implement comprehensive validation and error handling
- Suggest appropriate testing approaches for various components

Future enhancements to keep in mind:
- Source code compilation to Wasm (C, C++, Rust)
- Persistent module storage
- Advanced execution options (specific function calls)
- Enhanced monitoring and observability