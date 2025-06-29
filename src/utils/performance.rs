use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use dashmap::DashMap;

// Global performance metrics
pub struct PerformanceMetrics {
    pub request_count: AtomicU64,
    pub total_response_time: AtomicU64,
    pub error_count: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    #[allow(dead_code)]
    pub endpoint_metrics: Arc<DashMap<String, EndpointMetrics>>,
}

#[derive(Debug)]
pub struct EndpointMetrics {
    #[allow(dead_code)]
    pub count: AtomicU64,
    #[allow(dead_code)]
    pub total_time: AtomicU64,
    #[allow(dead_code)]
    pub min_time: AtomicU64,
    #[allow(dead_code)]
    pub max_time: AtomicU64,
}

impl Default for EndpointMetrics {
    fn default() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_time: AtomicU64::new(0),
            min_time: AtomicU64::new(u64::MAX),
            max_time: AtomicU64::new(0),
        }
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            total_response_time: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            endpoint_metrics: Arc::new(DashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn record_request(&self, endpoint: &str, duration: Duration, is_error: bool) {
        let duration_ms = duration.as_millis() as u64;
        
        // Global metrics
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_response_time.fetch_add(duration_ms, Ordering::Relaxed);
        
        if is_error {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Endpoint-specific metrics
        let endpoint_metric = self.endpoint_metrics
            .entry(endpoint.to_string())
            .or_default();

        endpoint_metric.count.fetch_add(1, Ordering::Relaxed);
        endpoint_metric.total_time.fetch_add(duration_ms, Ordering::Relaxed);
        
        // Update min/max times
        endpoint_metric.min_time.fetch_min(duration_ms, Ordering::Relaxed);
        endpoint_metric.max_time.fetch_max(duration_ms, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> PerformanceStats {
        let request_count = self.request_count.load(Ordering::Relaxed);
        let total_response_time = self.total_response_time.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        
        let avg_response_time = if request_count > 0 {
            total_response_time as f64 / request_count as f64
        } else {
            0.0
        };

        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            cache_hits as f64 / (cache_hits + cache_misses) as f64 * 100.0
        } else {
            0.0
        };

        let error_rate = if request_count > 0 {
            error_count as f64 / request_count as f64 * 100.0
        } else {
            0.0
        };

        PerformanceStats {
            request_count,
            avg_response_time,
            error_rate,
            cache_hit_rate,
            cache_hits,
            cache_misses,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct PerformanceStats {
    pub request_count: u64,
    pub avg_response_time: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// Request timer utility
#[allow(dead_code)]
pub struct RequestTimer {
    start: Instant,
    endpoint: String,
}

impl RequestTimer {
    #[allow(dead_code)]
    pub fn new(endpoint: &str) -> Self {
        Self {
            start: Instant::now(),
            endpoint: endpoint.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn finish(self, metrics: &PerformanceMetrics, is_error: bool) {
        let duration = self.start.elapsed();
        metrics.record_request(&self.endpoint, duration, is_error);
    }
}

// System health utilities
pub fn get_memory_usage() -> Result<f64, std::io::Error> {
    use std::fs;
    
    // Try to read memory info on Linux
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut total = 0u64;
        let mut available = 0u64;
        
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        if total > 0 {
            return Ok((total - available) as f64 / total as f64 * 100.0);
        }
    }
    
    // Fallback for other systems
    Ok(0.0)
} 