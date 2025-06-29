use actix_web::{web, HttpResponse, Result, Scope};
use sqlx::PgPool;
use crate::models::ApiResponse;
use crate::database::test_connection;
use crate::utils::performance::{PerformanceMetrics, get_memory_usage};

pub async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::<()>::success_no_data(
        "Job Finder Backend API is running"
    )))
}

pub async fn health_check(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match test_connection(&pool).await {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::<()>::success_no_data(
            "Service is healthy"
        ))),
        Err(_) => Ok(HttpResponse::ServiceUnavailable().json(ApiResponse::<()>::error(
            "Database connection failed"
        ))),
    }
}

pub async fn metrics(metrics: web::Data<PerformanceMetrics>) -> Result<HttpResponse> {
    let stats = metrics.get_stats();
    let memory_usage = get_memory_usage().unwrap_or(0.0);
    
    let health_info = serde_json::json!({
        "performance": stats,
        "system": {
            "memory_usage_percent": memory_usage,
            "uptime": "N/A" // Could be enhanced with actual uptime
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        "Performance metrics retrieved",
        health_info
    )))
}

pub fn health_routes() -> Scope {
    web::scope("/health")
        .route("", web::get().to(health_check))
        .route("/metrics", web::get().to(metrics))
} 