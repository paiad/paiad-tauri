// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


//stress-test
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::VecDeque;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkMetrics {
    pub timestamp: u64,
    pub latency: f64,
    pub packet_loss: f64,
    pub bandwidth: f64,
    pub is_anomaly: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct AnomalyDetectionResult {
    pub metrics: NetworkMetrics,
    pub anomaly_score: f64,
    pub threshold: f64,
}

struct NetworkMonitor {
    metrics_history: VecDeque<NetworkMetrics>,
    window_size: usize,
    anomaly_threshold: f64,
    client: Client,
}

impl NetworkMonitor {
    fn new(window_size: usize, anomaly_threshold: f64) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
        
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        NetworkMonitor {
            metrics_history: VecDeque::with_capacity(window_size),
            window_size,
            anomaly_threshold,
            client,
        }
    }

    async fn measure_latency(&self, url: &str) -> Result<f64, String> {
        let start = Instant::now();
        let client = self.client.clone();
        match client.get(url).send().await {
            Ok(_) => Ok(start.elapsed().as_millis() as f64),
            Err(e) => Err(format!("请求失败: {}", e)),
        }
    }

    async fn measure_packet_loss(&self, url: &str, packets: usize) -> Result<f64, String> {
        let mut lost = 0;
        let mut total = 0;
        let client = self.client.clone();

        for _ in 0..packets {
            total += 1;
            match client.get(url).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        lost += 1;
                    }
                }
                Err(_) => {
                    lost += 1;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok((lost as f64 / total as f64) * 100.0)
    }

    fn detect_anomaly(&mut self, mut metrics: NetworkMetrics) -> AnomalyDetectionResult { // <-- 1. 将 metrics 参数标记为 mut 可变
        let mut anomaly_score = 0.0;

        if self.metrics_history.len() > 0 {
            let avg_latency: f64 = self.metrics_history.iter()
                .map(|m| m.latency)
                .sum::<f64>() / self.metrics_history.len() as f64;

            let std_latency: f64 = (self.metrics_history.iter()
                .map(|m| (m.latency - avg_latency).powi(2))
                .sum::<f64>() / self.metrics_history.len() as f64)
                .sqrt();

            anomaly_score = (metrics.latency - avg_latency).abs() / (std_latency + 1e-10);
        }

        let is_anomaly = anomaly_score > self.anomaly_threshold;

        // **2. 核心修改：将计算出的 is_anomaly 赋值给 metrics 结构体中的字段**
        metrics.is_anomaly = is_anomaly;

        // 3. 更新历史记录：将更新后的 metrics 结构体添加到历史中
        //    这里我们使用更新后的 metrics 的克隆来添加到历史
        let metrics_to_history = metrics.clone();

        if self.metrics_history.len() >= self.window_size {
            self.metrics_history.pop_front();
        }
        self.metrics_history.push_back(metrics_to_history);


        AnomalyDetectionResult {
            metrics, // <-- 返回包含正确 is_anomaly 值的 metrics 结构体
            anomaly_score,
            threshold: self.anomaly_threshold,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MonitorConfig {
    pub window_size: usize,
    pub anomaly_threshold: f64,
    pub sampling_interval: u64,
}

#[tauri::command]
async fn run_load_test(config: LoadTestConfig) -> Result<LoadTestResult, String> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let mut handles = vec![];
    let mut durations = vec![];
    let mut success_count = 0usize;
    let mut fail_count = 0usize;

    let start = Instant::now();

    for _ in 0..config.total_requests {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let url = config.url.clone();
        let method = config.method.clone();

        let handle = tokio::spawn(async move {
            let req = match method.as_str() {
                "POST" => reqwest::Client::new().post(&url),
                _ => reqwest::Client::new().get(&url),
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

#[tauri::command]
async fn start_network_monitoring(
    config: MonitorConfig,
    state: tauri::State<'_, Mutex<NetworkMonitor>>,
) -> Result<(), String> {
    let mut monitor = state.lock().unwrap();
    *monitor = NetworkMonitor::new(config.window_size, config.anomaly_threshold);
    Ok(())
}

#[tauri::command]
async fn get_network_metrics(
    state: tauri::State<'_, Mutex<NetworkMonitor>>,
) -> Result<Vec<NetworkMetrics>, String> {
    let monitor = state.lock().unwrap();
    Ok(monitor.metrics_history.iter().cloned().collect())
}

#[tauri::command]
async fn update_metrics(
    metrics: NetworkMetrics,
    state: tauri::State<'_, Mutex<NetworkMonitor>>,
) -> Result<AnomalyDetectionResult, String> {
    let mut monitor = state.lock().unwrap();
    Ok(monitor.detect_anomaly(metrics))
}

#[tauri::command]
async fn measure_network_metrics(
    url: String,
    state: tauri::State<'_, Mutex<NetworkMonitor>>,
) -> Result<NetworkMetrics, String> {
    let client = {
        let monitor = state.lock().unwrap();
        monitor.client.clone()
    };
    
    let start = Instant::now();
    let latency = match client.get(&url).send().await {
        Ok(_) => start.elapsed().as_millis() as f64,
        Err(e) => return Err(format!("请求失败: {}", e)),
    };

    let mut lost = 0;
    let mut total = 0;
    for _ in 0..10 {
        total += 1;
        match client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    lost += 1;
                }
            }
            Err(_) => {
                lost += 1;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let packet_loss = (lost as f64 / total as f64) * 100.0;
    
    Ok(NetworkMetrics {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        latency,
        packet_loss,
        bandwidth: 0.0,
        is_anomaly: false,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(NetworkMonitor::new(100, 2.0)))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            run_load_test,
            start_network_monitoring,
            get_network_metrics,
            update_metrics,
            measure_network_metrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}