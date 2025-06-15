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

Git Workflow Best Practices:
- Always use atomic commits: `git add . && git commit -m "message"` to avoid staging issues
- Check status before committing: `git status` to verify what will be committed
- Use VS Code's "Commit All" button (checkmark with plus) for automatic staging and committing
- Prefer descriptive commit messages that explain what was implemented/changed
- When working on features, commit frequently with logical chunks of work
- Use `git commit --amend` sparingly and only for the most recent commit
- Set up useful git aliases:
  - `git config --global alias.ac '!git add . && git commit -m'` for quick commits
  - `git config --global alias.s 'status'` for quick status checks

File Editing Best Practices:
- When using insert_edit_into_file tool, avoid repeating existing code
- Use `// ...existing code...` comments to represent unchanged regions
- When using replace_string_in_file tool, include 3-5 lines of context before and after the target string
- This ensures unambiguous identification of what should be edited
- Make targeted, precise edits rather than large rewrites when possible
- Always verify the change makes sense in the broader context of the file