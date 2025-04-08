use actix_msgpack::MsgPack;
use actix_web::{post, web::Data, HttpResponse, Responder};
use log::{trace, warn, error, info};
use serde::Deserialize;
use std::{fs::{OpenOptions, self}, io::Write, path::PathBuf, sync::Arc};
use serde_json;

use crate::core::Core;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LogRequest {
    client_id: Option<u32>,
    timestamp: i64,
    message: String,
    #[serde(rename = "type")]
    level: String,
}

fn get_log_path() -> PathBuf {
    // Always use the current working directory which should be the project root
    match std::env::current_dir() {
        Ok(mut path) => {
            path.push("lemonlogs.txt");
            info!("Using absolute log path: {:?}", path);
            path
        },
        Err(err) => {
            // Fallback to relative path if current_dir fails
            warn!("Failed to get current working directory: {}", err);
            let mut path = PathBuf::from(".");
            path.push("lemonlogs.txt");
            warn!("Using fallback relative path: {:?}", path);
            path
        }
    }
}

fn clear_log_file(log_path: &PathBuf, client_id_str: &str) -> bool {
    let path_str = log_path.to_string_lossy();
    info!("Clearing log file: {}", path_str);

    // Ensure directory exists
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            match std::fs::create_dir_all(parent) {
                Ok(_) => info!("Created parent directory for log file"),
                Err(e) => warn!("Failed to create parent directory: {}", e),
            }
        }
    }
    
    // Always try to create/truncate
    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(log_path)
    {
        Ok(mut file) => {
            info!("Successfully cleared/created log file: {}", path_str);
            let starting_log = format!(
                "[{}] [INFO] === New game session started (triggered by client: {}) ===\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                client_id_str
            );
            match file.write_all(starting_log.as_bytes()) {
                Ok(_) => {
                    let _ = file.flush(); // Attempt flush, ignore error
                    info!("Wrote session start marker.");
                    true // Indicate success
                },
                Err(e) => {
                    warn!("Failed to write initial message: {}", e);
                    false // Indicate failure
                }
            }
        },
        Err(err) => {
            error!("Failed to clear/create log file: {}", err);
            false // Indicate failure
        }
    }
}

#[post("/log")]
async fn main(request: MsgPack<LogRequest>, _core: Data<Arc<Core>>) -> impl Responder {
    // Log the raw request first for detailed debugging
    trace!("RAW /log request: {:#?}", request);

    let log_request_data = request.0;
    let log_path = get_log_path();
    let client_id_str = log_request_data.client_id.map_or("Unknown".to_string(), |id| id.to_string());

    // Check if this is the game start trigger message
    const GAME_START_MARKER: &str = "[GAME START DETECTOR] Game is running, firing event...";
    
    // !!! Log the exact message received right before comparison !!!
    info!("Received message for /log: \"{}\"", log_request_data.message);
    info!("Comparing with marker:  \"{}\"", GAME_START_MARKER);
    
    if log_request_data.message == GAME_START_MARKER {
        info!("✅ MATCH FOUND! Triggering log clear for client: {}", client_id_str);
        
        let success = clear_log_file(&log_path, &client_id_str);
        
        if success {
             HttpResponse::Ok().body(format!(
                "Log cleared successfully at {} for client {}", 
                chrono::Local::now().format("%H:%M:%S"),
                client_id_str
            ))
        } else {
            HttpResponse::InternalServerError().body("Failed to clear log file")
        }
    } else {
        info!("❌ No match. Appending as regular log.");
        // Proceed with appending the regular log message
        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            if !parent.exists() {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    error!("Failed to create log directory: {}", err);
                    // Proceed anyway, open might create it
                }
            }
        }

        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(file) => file,
            Err(err) => {
                error!("Failed to open log file {:?}: {}", log_path, err);
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to open log file: {}", err));
            }
        };

        let log_entry = format!(
            "[{}] [{}] {}\n",
            chrono::DateTime::from_timestamp(log_request_data.timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| log_request_data.timestamp.to_string()),
            log_request_data.level,
            log_request_data.message
        );

        match file.write_all(log_entry.as_bytes()) {
            Ok(_) => {
                let _ = file.flush(); // Attempt flush, ignore error
                HttpResponse::Ok().body("Log written successfully")
            },
            Err(err) => {
                error!("Failed to write to log file {:?}: {}", log_path, err);
                HttpResponse::InternalServerError().body(format!("Failed to write log: {}", err))
            }
        }
    }
} 