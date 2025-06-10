
## **Engineering Requirements Document (ERD): WebAssembly Compilation and Execution API**

Version: 1.0

Date: June 10, 2025

---

### **1. Introduction**

This document outlines the detailed engineering requirements for the WebAssembly Compilation and Execution API. The API will be a high-performance, secure, and scalable service built in Rust, leveraging the Actix-web framework and Wasmer runtime. The initial Minimum Viable Product (MVP) focuses on the execution of pre-compiled WebAssembly (Wasm) modules, with future plans to incorporate on-demand compilation from source code (C, C++, Rust).

### **2. Architecture Overview**

The API will be implemented as a monolithic Rust application using the Actix-web framework. It will interact with a PostgreSQL database for authentication and usage tracking, and temporarily store Wasm modules on the file system for execution. The core execution will be handled by the Wasmer runtime within a sandboxed WebAssembly System Interface (WASI) environment.

**Key Components:**

- **Web Server (Actix-web):** Handles HTTP requests, authentication, and rate limiting.
- **Execution Engine (Wasmer):** Executes Wasm modules in a sandboxed environment using WASI.
- **Storage:** Temporarily stores Wasm modules on the file system for execution.
- **Authentication & Rate Limiting Middleware:** Secures the API and manages usage based on subscription tiers.
- **PostgreSQL Database:** Stores API keys and usage data.
- **Web Interface (SSR):** A simple frontend for users to test the API.

### **3. API Endpoints (MVP)**

The MVP will focus on a single endpoint for executing pre-compiled Wasm modules.

#### **3.1. POST /execute**

- **Purpose:** Execute a user-submitted Wasm module with provided input and return the output.
- **Request Format:** Multipart form data.
    - `wasm`: The Wasm module as a binary file.
        - **Requirement:** Max file size: 10 MB.
        - **Validation:** Must be a valid `.wasm` binary. Invalid Wasm will result in a `400 Bad Request`.
    - `input`: A string representing the input to the Wasm module.
        - **Requirement:** Max string size: 1 MB.
- **Request Headers:**
    - `Authorization`: `Bearer <API_KEY>` (API key for authentication).
- **Response Format:** JSON
    - `output` (string): The standard output (stdout) captured from the Wasm module's execution.
    - `error` (string, nullable): Any error messages from the execution, or `null` if successful. User-friendly; no raw stack traces exposed.
- **HTTP Status Codes:**
    - `200 OK`: Successful execution.
    - `400 Bad Request`: Invalid request format (e.g., missing parts, malformed multipart data, invalid Wasm binary).
    - `401 Unauthorized`: Invalid or missing API key.
    - `403 Forbidden`: API key valid but unauthorized (e.g., rate limit exceeded for the requested action).
    - `422 Unprocessable Entity`: Wasm execution failed due to an application-level error within the Wasm module (e.g., WASI trap, resource limit hit). The `error` field will provide details.
    - `500 Internal Server Error`: Unexpected server-side issue.

### **4. Functional Requirements**

#### **4.1. Wasm Module Handling and Execution**

- **FR.4.1.1 - Module Upload:** The API SHALL accept Wasm modules as binary files via multipart form data.
- **FR.4.1.2 - Temporary Storage:** The API SHALL save uploaded Wasm modules temporarily on the server's file system (e.g., `/tmp/wasm_modules`).
    - **FR.4.1.2.1 - Unique ID:** Each module SHALL be saved with a unique ID (UUID v4) as its filename (e.g., `/tmp/wasm_modules/<UUID>.wasm`).
    - **FR.4.1.2.2 - Cleanup:** A Time-to-Live (TTL) mechanism SHALL delete modules after 1 hour.
- **FR.4.1.3 - Wasm Loading:** The API SHALL use the `wasmer` crate to load the Wasm module from the temporary file.
- **FR.4.1.4 - WASI Environment Setup:** The API SHALL set up a WASI environment for the Wasm module execution.
    - **FR.4.1.4.1 - Input Provisioning:** The WASI environment SHALL read input from the provided `input` string, mapping it to stdin.
    - **FR.4.1.4.2 - Output Capture:** The WASI environment SHALL capture output written to stdout by the Wasm module.
- **FR.4.1.5 - Execution Entry Point:** The API SHALL execute the Wasm module's `_start` entry point.
- **FR.4.1.6 - Result Return:** The API SHALL return the captured stdout as `output` or any runtime errors as `error` in the JSON response.

#### **4.2. Security and Sandboxing**

- **FR.4.2.1 - Sandboxed Execution:** The API SHALL execute Wasm modules within a secure, isolated sandbox using Wasmer's WASI implementation.
- **FR.4.2.2 - File System Access Control:** The sandboxed environment SHALL disable file system access, except for standard I/O (stdin/stdout).
- **FR.4.2.3 - Network Access Control:** The sandboxed environment SHALL prevent all network access.
- **FR.4.2.4 - Time-Based Resource Limit:** The API SHALL enforce a maximum execution time of 5 seconds per Wasm module execution.
    - **FR.4.2.4.1 - Termination:** Exceeding the time limit SHALL result in immediate termination of the Wasm module.
- **FR.4.2.5 - Memory Usage Limit:** The API SHALL enforce a maximum memory usage of 128 MB per Wasm module execution.
    - **FR.4.2.5.1 - Termination:** Exceeding the memory limit SHALL result in immediate termination of the Wasm module.
- **FR.4.2.6 - WASI Capability Restrictions:** The WASI environment SHALL explicitly disable `clocks`, `env` (environment variables), and `rand` (random number generation) capabilities.

#### **4.3. Authentication**

- **FR.4.3.1 - API Key Authentication:** The API SHALL implement API key-based authentication using Actix-web middleware.
- **FR.4.3.2 - Key Storage:** API keys SHALL be stored as SHA-256 hashes in the PostgreSQL database.
- **FR.4.3.3 - Key Validation:** Each API request SHALL be validated against the stored API key hashes.
- **FR.4.3.4 - Key Generation:** Users SHALL be able to generate API keys via the web interface. Newly created user accounts SHALL automatically receive one API key.

#### **4.4. Rate Limiting and Usage Tracking**

- **FR.4.4.1 - Rate Limiting Enforcement:** The API SHALL limit the number of requests per API key based on subscription tiers, using a token bucket algorithm.
    - **Free Tier:** 10 executions per minute, 500 executions per day.
    - **Basic Tier:** 100 executions per minute, 10,000 executions per day.
    - **Pro Tier:** 500 executions per minute, 50,000 executions per day.
- **FR.4.4.2 - Rate Limit Exceeded Response:** Upon exceeding a rate limit, the API SHALL return a `429 Too Many Requests` HTTP status code with a `Retry-After` header.
- **FR.4.4.3 - Usage Logging:** The API SHALL log usage data in the PostgreSQL database for billing and analytics.
    - **FR.4.4.3.1 - Granularity:** Logged data SHALL include API key ID, timestamp, execution duration, peak memory usage, execution status (success/failure), error message (if any), Wasm module size, and input size.

#### **4.5. Web Interface**

- **FR.4.5.1 - Module Upload Form:** The web interface SHALL provide a form for users to upload a Wasm module file.
    - **FR.4.5.1.1 - Client-side Validation:** Implement client-side validation for `.wasm` file extension and file size (max 10 MB).
- **FR.4.5.2 - Input Text Area:** The web interface SHALL provide a text area for users to enter input data.
    - **FR.4.5.2.1 - Client-side Validation:** Implement client-side validation for input text size (max 1 MB).
- **FR.4.5.3 - Output Display:** The web interface SHALL display the execution output and any error messages in a dedicated area.
- **FR.4.5.4 - User Feedback:** The web interface SHALL provide visual feedback during processing (e.g., loading spinners) and clear success/error messages.
- **FR.4.5.5 - API Key Management:** The web interface SHALL allow users to generate new API keys.

### **5. Non-Functional Requirements**

#### **5.1. Performance**

- **NFR.5.1.1 - Response Time:** The API SHALL aim for sub-200ms response times for typical Wasm executions (excluding network latency and module transfer time).
- **NFR.5.1.2 - Concurrency:** The API SHALL be able to handle at least 50 concurrent Wasm executions.
- **NFR.5.1.3 - Throughput:** The API SHALL be capable of processing requests adhering to the specified rate limits across all active users.

#### **5.2. Scalability**

- **NFR.5.2.1 - Horizontal Scaling:** The application SHALL be designed for horizontal scaling, allowing additional instances to be added behind a load balancer.
- **NFR.5.2.2 - Database Scalability:** The PostgreSQL database SHALL be configured for future scaling (e.g., read replicas, connection pooling).

#### **5.3. Security**

- **NFR.5.3.1 - Data Protection:** API keys and sensitive user data in the database SHALL be protected using hashing or encryption.
- **NFR.5.3.2 - Input Validation:** All API inputs SHALL be rigorously validated to prevent injection attacks and other vulnerabilities.
- **NFR.5.3.3 - Sandbox Integrity:** The Wasm sandboxing SHALL be continuously reviewed and tested to ensure isolation.
- **NFR.5.3.4 - Secure Communication:** All API communication SHALL occur over HTTPS/TLS.

#### **5.4. Reliability and Availability**

- **NFR.5.4.1 - Error Handling:** The API SHALL gracefully handle errors, providing informative responses without crashing.
- **NFR.5.4.2 - Uptime:** The target uptime for the service is 99.9%.

#### **5.5. Maintainability and Extensibility**

- **NFR.5.5.1 - Code Quality:** The codebase SHALL adhere to Rust best practices, including clear module separation, comprehensive unit tests, and idiomatic Rust.
- **NFR.5.5.2 - Modularity:** The architecture SHALL be modular to facilitate future enhancements (e.g., compilation feature, advanced execution options) without significant re-architecting.
- **NFR.5.5.3 - Configuration:** Key parameters (e.g., resource limits, temporary storage paths, database connection strings) SHALL be configurable via environment variables.

### **6. Technology Stack**

- **Programming Language:** Rust
- **Web Framework:** Actix-web
    - **Dependencies:** `actix-multipart`, `serde`, `uuid`, `tokio`
- **Wasm Runtime:** Wasmer
    - **Dependencies:** `wasmer-wasi`
- **Database:** PostgreSQL
    - **ORM/SQL Client:** `sqlx` (with connection pooling)
- **Templating Engine (Web UI):** Askama
- **Logging:** `tracing` crate
- **Containerization:** Docker

### **7. Deployment Requirements**

- **DR.7.1 - Container Image:** The application SHALL be deployed as a Docker container image.
    - **DR.7.1.1 - Multi-stage Build:** The Dockerfile SHALL use a multi-stage build to create a small, optimized production image.
    - **DR.7.1.2 - Minimal Base Image:** The final image SHALL use a minimal base image (e.g., `distroless/cc-debian11` or `alpine`).
    - **DR.7.1.3 - Non-root User:** The application process within the container SHALL run as a non-root user.
- **DR.7.2 - Environment Configuration:** All sensitive configurations (database credentials, API salts) SHALL be passed as environment variables to the container.
- **DR.7.3 - Logging:** Container logs (stdout/stderr) SHALL be collected by an external logging system.

### **8. Future Enhancements (High-Level Roadmap)**

These are planned features beyond the MVP.

- **Compilation Feature (POST /compile):**
    - Accept source code (C, C++, Rust).
    - Compile to Wasm using Emscripten (for C/C++) and `wasm-pack` (for Rust).
    - Compilation tools to run in isolated Docker containers per job.
    - Return compiled module or identifier.
- **Advanced Execution Options:**
    - Allow specifying which function to call within a Wasm module with arguments.
- **Persistent Modules:**
    - Enable users to store and manage their compiled modules for repeated executions.
    - Utilize cloud object storage (e.g., S3-compatible).
    - Provide API endpoints and UI for module management (list, delete, update).
- **Enhanced Monitoring:** Implement Prometheus and Grafana for detailed performance monitoring and alerting.

### **9. Testing and Quality Assurance**

- **TQA.9.1 - Unit Tests:** Comprehensive unit tests for individual components (e.g., Wasm execution logic, authentication middleware, database interactions).
- **TQA.9.2 - Integration Tests:** End-to-end integration tests simulating API requests with sample Wasm modules, including valid and invalid scenarios.
- **TQA.9.3 - Security Tests:** Dedicated tests to verify sandboxing integrity by attempting to execute malicious or resource-intensive Wasm modules.
- **TQA.9.4 - Performance Tests:** Load tests to measure response times and resource usage under various loads to ensure NFRs are met.

### **10. Documentation**

- **Doc.10.1 - API Documentation:** Generate interactive API documentation using OpenAPI (Swagger).
- **Doc.10.2 - Code Examples:** Provide `curl` commands and code snippets for popular languages (Rust, C/C++, Python, JavaScript, Go) for API usage examples.
- **Doc.10.3 - Quick Start Guides:** Develop clear, step-by-step quick start guides for common use cases.
- **Doc.10.4 - Error Reference:** Document all possible error codes and their meanings.

---

This ERD should provide your development team with a clear, concise, and comprehensive set of requirements for building the WebAssembly Compilation and Execution API. Good luck with the development process!