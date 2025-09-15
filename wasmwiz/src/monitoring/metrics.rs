// src/monitoring/metrics.rs
// Prometheus metrics collection for Wasm Wizard

use once_cell::sync::Lazy;
use prometheus::{Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry};
use std::sync::Arc;
use tracing::{debug, error};

pub struct Metrics {
    // Request metrics
    pub http_requests_total: IntCounter,
    pub http_request_duration: Histogram,
    pub active_connections: IntGauge,

    // WASM execution metrics
    pub wasm_executions_total: IntCounter,
    pub wasm_execution_duration: Histogram,
    pub wasm_execution_errors: IntCounter,
    pub wasm_memory_usage: Histogram,

    // Rate limiting metrics
    pub rate_limit_hits: IntCounter,
    pub rate_limit_violations: IntCounter,

    // System metrics
    pub system_memory_usage: Gauge,
    pub system_cpu_usage: Gauge,

    registry: Arc<Registry>,
}

impl Metrics {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Arc::new(Registry::new());

        // HTTP metrics
        let http_requests_total = IntCounter::with_opts(Opts::new(
            "http_requests_total",
            "Total number of HTTP requests",
        ))?;
        registry.register(Box::new(http_requests_total.clone()))?;

        let http_request_duration = Histogram::with_opts(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        )?;
        registry.register(Box::new(http_request_duration.clone()))?;

        let active_connections =
            IntGauge::with_opts(Opts::new("active_connections", "Number of active connections"))?;
        registry.register(Box::new(active_connections.clone()))?;

        // WASM execution metrics
        let wasm_executions_total = IntCounter::with_opts(Opts::new(
            "wasm_executions_total",
            "Total number of WASM executions",
        ))?;
        registry.register(Box::new(wasm_executions_total.clone()))?;

        let wasm_execution_duration = Histogram::with_opts(
            HistogramOpts::new(
                "wasm_execution_duration_seconds",
                "WASM execution duration in seconds",
            )
            .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
        )?;
        registry.register(Box::new(wasm_execution_duration.clone()))?;

        let wasm_execution_errors = IntCounter::with_opts(Opts::new(
            "wasm_execution_errors_total",
            "Total number of WASM execution errors",
        ))?;
        registry.register(Box::new(wasm_execution_errors.clone()))?;

        let wasm_memory_usage = Histogram::with_opts(
            HistogramOpts::new("wasm_memory_usage_bytes", "WASM execution memory usage in bytes")
                .buckets(vec![
                    1024.0,
                    10240.0,
                    102400.0,
                    1048576.0,
                    10485760.0,
                    104857600.0,
                ]),
        )?;
        registry.register(Box::new(wasm_memory_usage.clone()))?;

        // Rate limiting metrics
        let rate_limit_hits = IntCounter::with_opts(Opts::new(
            "rate_limit_hits_total",
            "Total number of rate limit checks",
        ))?;
        registry.register(Box::new(rate_limit_hits.clone()))?;

        let rate_limit_violations = IntCounter::with_opts(Opts::new(
            "rate_limit_violations_total",
            "Total number of rate limit violations",
        ))?;
        registry.register(Box::new(rate_limit_violations.clone()))?;

        // System metrics
        let system_memory_usage = Gauge::with_opts(Opts::new(
            "system_memory_usage_ratio",
            "System memory usage ratio (0-1)",
        ))?;
        registry.register(Box::new(system_memory_usage.clone()))?;

        let system_cpu_usage =
            Gauge::with_opts(Opts::new("system_cpu_usage_ratio", "System CPU usage ratio (0-1)"))?;
        registry.register(Box::new(system_cpu_usage.clone()))?;

        Ok(Self {
            http_requests_total,
            http_request_duration,
            active_connections,
            wasm_executions_total,
            wasm_execution_duration,
            wasm_execution_errors,
            wasm_memory_usage,
            rate_limit_hits,
            rate_limit_violations,
            system_memory_usage,
            system_cpu_usage,
            registry,
        })
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }

    // Helper methods for recording metrics
    pub fn record_wasm_execution(&self, duration: f64, memory_used: f64, success: bool) {
        self.wasm_executions_total.inc();
        self.wasm_execution_duration.observe(duration);
        self.wasm_memory_usage.observe(memory_used);

        if !success {
            self.wasm_execution_errors.inc();
        }

        debug!(
            "Recorded WASM execution: duration={}s, memory={}MB, success={}",
            duration,
            memory_used / 1024.0 / 1024.0,
            success
        );
    }

    pub fn record_rate_limit_check(&self, violated: bool) {
        self.rate_limit_hits.inc();
        if violated {
            self.rate_limit_violations.inc();
        }
    }

    pub fn update_system_metrics(&self) {
        use sysinfo::System;
        let mut system = System::new_all();
        system.refresh_all();

        // Memory usage ratio
        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        if total_memory > 0.0 {
            let memory_ratio = used_memory / total_memory;
            self.system_memory_usage.set(memory_ratio);
        }

        // CPU usage (average across all cores)
        let cpu_usage: f64 = system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64 / 100.0)
            .sum::<f64>()
            / system.cpus().len() as f64;
        self.system_cpu_usage.set(cpu_usage);
    }
}

// Global metrics instance
pub static METRICS: Lazy<Metrics> = Lazy::new(|| {
    Metrics::new().unwrap_or_else(|e| {
        error!("Failed to initialize metrics: {}", e);
        panic!("Metrics initialization failed")
    })
});

// Middleware for collecting HTTP metrics
use actix_web::dev::{ServiceResponse, Transform};
use actix_web::{dev::ServiceRequest, Error, Result};
use futures_util::future::{ok, Ready};
use std::time::Instant;

pub struct MetricsMiddleware;

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = MetricsMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddlewareService { service })
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        METRICS.active_connections.inc();

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;

            let duration = start_time.elapsed().as_secs_f64();
            METRICS.http_requests_total.inc();
            METRICS.http_request_duration.observe(duration);
            METRICS.active_connections.dec();

            result
        })
    }
}
