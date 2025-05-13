// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


//stress-test
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoadTestConfig {
    pub url: String,
    pub method: String,
    pub concurrency: usize,
    pub total_requests: usize,
}

#[derive(Debug, Serialize)]
pub struct LoadTestResult {
    pub total: usize,
    pub success: usize,
    pub fail: usize,
    pub avg_time_ms: f64,
}

#[tauri::command]
async fn run_load_test(config: LoadTestConfig) -> Result<LoadTestResult, String> {
    use reqwest::Client;
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let mut handles = vec![];
    let mut durations = vec![];
    let mut success_count = 0usize;
    let mut fail_count = 0usize;

    let start = Instant::now();

    for _ in 0..config.total_requests {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let url = config.url.clone();
        let method = config.method.clone();

        let handle = tokio::spawn(async move {
            let req = match method.as_str() {
                "POST" => client.post(&url),
                _ => client.get(&url),
            };

            let req_start = Instant::now();
            let result = req.send().await;
            let duration = req_start.elapsed().as_millis();

            drop(permit);
            (result.is_ok(), duration)
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Ok((ok, duration)) = handle.await {
            durations.push(duration as f64);
            if ok {
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
    }

    let avg_time = if durations.len() > 0 {
        durations.iter().sum::<f64>() / durations.len() as f64
    } else {
        0.0
    };

    Ok(LoadTestResult {
        total: config.total_requests,
        success: success_count,
        fail: fail_count,
        avg_time_ms: avg_time,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet,run_load_test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}