use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::{Logger, Compress, DefaultHeaders}};
use std::time::Duration;
use std::net::{TcpListener, SocketAddr};

// Use faster memory allocator
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod config;
mod cache;
mod database;
mod handlers;
mod models;
mod middleware; // Re-enabled middleware
mod utils;

use config::Config;
use database::create_pool;
use cache::CacheManager;

/// Find an available port starting from the given port
fn find_available_port(host: &str, start_port: u16) -> u16 {
    for port in start_port..start_port + 100 {
        let addr = format!("{host}:{port}");
        if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
            if TcpListener::bind(socket_addr).is_ok() {
                return port;
            }
        }
    }
    start_port // Fallback to original port
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment variables
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    
    // Create database connection pool with optimized settings
    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");
    
    // Initialize cache manager for performance optimization
    let cache_manager = CacheManager::new();
    
    // Run database migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    
    let host = config.host.clone();
    let configured_port = config.port;
    
    // Find an available port
    let available_port = find_available_port(&host, configured_port);
    
    if available_port != configured_port {
        log::warn!("Port {configured_port} is in use, switching to port {available_port}");
    }
    
    log::info!("Starting server at {host}:{available_port}");
    
    // Create HTTP server with performance optimizations
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // Frontend origin
            .allowed_origin("http://127.0.0.1:3000")
            .allow_any_origin() // Allow requests from mobile apps and other clients
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(cache_manager.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Compress::default()) // Enable compression for all responses
            .wrap(DefaultHeaders::new()
                .add(("X-Version", "1.0"))
                .add(("Cache-Control", "public, max-age=3600")) // Cache static resources
            )
            .service(
                web::scope("/api/v1")
                    .service(handlers::auth::auth_routes())
                    .service(
                        web::scope("")
                            .wrap(middleware::AuthMiddleware)
                            .service(handlers::users::user_routes())
                    )
                    .service(handlers::health::health_routes())
            )
            .service(
                web::scope("/")
                    .route("/", web::get().to(handlers::health::index))
            )
    })
    .workers(num_cpus::get()) // Use all available CPU cores
    .max_connections(25000)   // High connection limit
    .client_request_timeout(Duration::from_secs(5))     // 5 second timeout
    .client_disconnect_timeout(Duration::from_secs(1)) // 1 second disconnect timeout
    .bind(format!("{host}:{available_port}"))?
    .run()
    .await
}
