//! # Utility Functions
//!
//! This module contains common utility functions and helpers used throughout
//! the Wasm Wizard application.
//!
//! ## Utilities Overview
//!
//! - **`auth`**: Authentication-related utility functions
//! - **`file_system`**: File system operations and temporary file management
//!
//! ## Design Principles
//!
//! - **Pure Functions**: Utilities should be stateless and side-effect free where possible
//! - **Error Handling**: All utilities return `Result` types with appropriate error messages
//! - **Async Support**: File operations are asynchronous for scalability
//! - **Security**: File operations include security checks and validation
//!
//! ## Usage
//!
//! ```rust,no_run
//! use wasm-wizard::utils::{auth, file_system};
//!
//! // Authentication utilities
//! let hashed_key = auth::hash_api_key("user-key", "salt")?;
//!
//! // File system utilities
//! let temp_file = file_system::create_temp_file().await?;
//! file_system::cleanup_temp_files().await?;
//! ```

pub mod auth;
pub mod file_system;
// pub mod rate_limit; // If you build your own custom rate limiter logic here
