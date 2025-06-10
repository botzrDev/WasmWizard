// src/utils/file_system.rs
// Utilities for handling temporary files.

use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;
use chrono::{Utc, Duration, DateTime};
use tracing::{info, error}; // Using tracing for logging

#[allow(dead_code)]
/// Gets the base directory for temporary Wasm modules.
/// Ensures the directory exists.
pub async fn get_wasm_temp_dir() -> Result<PathBuf, std::io::Error> {
    let temp_dir = PathBuf::from("/tmp/wasm_modules"); //
    fs::create_dir_all(&temp_dir).await?;
    Ok(temp_dir)
}

#[allow(dead_code)]
/// Creates a unique temporary file path for a Wasm module.
pub async fn create_unique_wasm_file_path() -> Result<PathBuf, std::io::Error> {
    let temp_dir = get_wasm_temp_dir().await?;
    let file_name = format!("{}.wasm", Uuid::new_v4()); //
    Ok(temp_dir.join(file_name))
}

#[allow(dead_code)]
/// Initiates a background task to clean up old Wasm modules.
/// This is a simplified example; a robust solution might involve a separate worker or cron job.
pub fn start_wasm_cleanup_task() {
    tokio::spawn(async move {
        info!("Wasm cleanup task started.");
        let temp_dir = match get_wasm_temp_dir().await {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to get Wasm temp directory for cleanup: {}", e);
                return;
            }
        };

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await; // Run every hour
            info!("Running Wasm cleanup task...");
            let entries = match fs::read_dir(&temp_dir).await {
                Ok(read_dir) => read_dir,
                Err(e) => {
                    error!("Failed to read Wasm temp directory for cleanup: {}", e);
                    continue;
                }
            };

            let now = Utc::now();
            let cutoff = now - Duration::hours(1); // Modules older than 1 hour

            tokio::pin!(entries); // Pin the stream for `next_entry`

            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        if let Ok(modified_time) = metadata.modified() {
                            let modified_time_utc: DateTime<Utc> = modified_time.into();
                            if modified_time_utc < cutoff {
                                if let Err(e) = fs::remove_file(&path).await {
                                    error!("Failed to remove old Wasm module {}: {}", path.display(), e);
                                } else {
                                    info!("Cleaned up old Wasm module: {}", path.display());
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}