use actix_web::{web, HttpResponse, Result, Scope};
use sqlx::{PgPool, Row};
use crate::models::{
    ApiResponse, LoginRequest, LoginResponse, CreateUserRequest, User, UserPublic, UserSession
};
use crate::utils::{hash_password, verify_password, generate_jwt};
use crate::config::Config;
use crate::cache::CacheManager;
use crate::database;
use chrono::{Duration, Utc};
use uuid::Uuid;

pub async fn register(
    pool: web::Data<PgPool>,
    cache: web::Data<CacheManager>,
    config: web::Data<Config>,
    user_data: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    // Input validation
    if user_data.username.trim().is_empty() || user_data.username.len() < 3 {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Username must be at least 3 characters long"
        )));
    }
    
    if user_data.email.trim().is_empty() || !user_data.email.contains('@') {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Valid email is required"
        )));
    }
    
    if user_data.password.len() < 8 {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Password must be at least 8 characters long"
        )));
    }
    
    if let Some(role) = &user_data.role {
        if !crate::models::user::is_valid_role(role) {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Role must be one of: job_seeker, employer, admin"
            )));
        }
    }

    // Hash password
    let password_hash = match hash_password(&user_data.password) {
        Ok(hash) => hash,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to hash password"
            )));
        }
    };

    // Insert user into database
    let user_result = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, first_name, last_name, phone, role, professional_role, company_name)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, username, email, password_hash, first_name, last_name, phone, role, professional_role, company_name, is_active, email_verified, created_at, updated_at
        "#
    )
    .bind(&user_data.username)
    .bind(&user_data.email)
    .bind(&password_hash)
    .bind(&user_data.first_name)
    .bind(&user_data.last_name)
    .bind(&user_data.phone)
    .bind(user_data.role.as_deref().unwrap_or("job_seeker"))
    .bind(&user_data.professional_role)
    .bind(&user_data.company_name)
    .fetch_one(pool.get_ref())
    .await;

    match user_result {
        Ok(user) => {
            // Generate JWT token for automatic login
            match generate_jwt(&user, &config.jwt_secret, config.jwt_expiration) {
                Ok(token) => {
                    // Create session in database
                    let session_token = Uuid::new_v4().to_string();
                    let expires_at = Utc::now() + Duration::seconds(config.jwt_expiration);
                    
                    let session_result = sqlx::query_as::<_, UserSession>(
                        r#"
                        INSERT INTO user_sessions (user_id, session_token, expires_at)
                        VALUES ($1, $2, $3)
                        RETURNING id, user_id, session_token, expires_at, created_at
                        "#
                    )
                    .bind(user.id)
                    .bind(&session_token)
                    .bind(expires_at)
                    .fetch_one(pool.get_ref())
                    .await;

                    match session_result {
                        Ok(_) => {
                            let user_public: UserPublic = user.into();
                            
                            // Cache the new user for future requests
                            cache.cache_user(user_public.clone()).await;
                            
                            // Return LoginResponse instead of just UserPublic
                            let response = LoginResponse {
                                access_token: token,
                                token_type: "Bearer".to_string(),
                                expires_in: config.jwt_expiration,
                                user: user_public,
                            };

                            Ok(HttpResponse::Created().json(ApiResponse::success(
                                "User registered and logged in successfully",
                                response
                            )))
                        }
                        Err(_) => {
                            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                                "Failed to create session"
                            )))
                        }
                    }
                }
                Err(_) => {
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        "Failed to generate token"
                    )))
                }
            }
        }
        Err(sqlx::Error::Database(db_err)) if db_err.constraint().is_some() => {
            Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                "Username or email already exists"
            )))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to create user"
            )))
        }
    }
}

pub async fn login(
    pool: web::Data<PgPool>,
    _config: web::Data<Config>,
    login_data: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    // Input validation
    if login_data.username_or_email.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Username or email is required"
        )));
    }
    
    if login_data.password.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Password is required"
        )));
    }

    // Use optimized database query
    let user_result = database::get_user_by_email_or_username_optimized(&pool, &login_data.username_or_email).await;

    let user = match user_result {
        Ok(Some(row)) => {
            User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                phone: row.get("phone"),
                role: row.get("role"),
                professional_role: row.get("professional_role"),
                company_name: row.get("company_name"),
                is_active: row.get("is_active"),
                email_verified: row.get("email_verified"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Invalid credentials"
            )));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error"
            )));
        }
    };

    // Check if user is active
    if !user.is_active {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            "Account is deactivated"
        )));
    }

    // Verify password
    match verify_password(&login_data.password, &user.password_hash) {
        Ok(true) => {
            // Generate JWT token
            match generate_jwt(&user, &_config.jwt_secret, _config.jwt_expiration) {
                Ok(token) => {
                    // Create session in database
                    let session_token = Uuid::new_v4().to_string();
                    let expires_at = Utc::now() + Duration::seconds(_config.jwt_expiration);
                    
                    let session_result = sqlx::query_as::<_, UserSession>(
                        r#"
                        INSERT INTO user_sessions (user_id, session_token, expires_at)
                        VALUES ($1, $2, $3)
                        RETURNING id, user_id, session_token, expires_at, created_at
                        "#
                    )
                    .bind(user.id)
                    .bind(&session_token)
                    .bind(expires_at)
                    .fetch_one(pool.get_ref())
                    .await;

                    match session_result {
                        Ok(_) => {
                            let user_public: UserPublic = user.into();
                            let response = LoginResponse {
                                access_token: token,
                                token_type: "Bearer".to_string(),
                                expires_in: _config.jwt_expiration,
                                user: user_public,
                            };

                            Ok(HttpResponse::Ok().json(ApiResponse::success(
                                "Login successful",
                                response
                            )))
                        }
                        Err(_) => {
                            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                                "Failed to create session"
                            )))
                        }
                    }
                }
                Err(_) => {
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        "Failed to generate token"
                    )))
                }
            }
        }
        Ok(false) => {
            Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Invalid credentials"
            )))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Password verification failed"
            )))
        }
    }
}

// Updated logout endpoint to clean up sessions
pub async fn logout(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    cache: web::Data<CacheManager>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse> {
    // Extract token from Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Add token to blacklist
                cache.blacklist_token(token.to_string()).await;
                
                // Extract user claims to get user_id for session cleanup
                if let Ok(claims) = crate::utils::jwt::decode_jwt(token, &config.jwt_secret) {
                    if let Ok(user_id) = claims.sub.parse::<i32>() {
                        // Delete user sessions from database
                        let _ = sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
                            .bind(user_id)
                            .execute(pool.get_ref())
                            .await;
                    }
                }
                
                return Ok(HttpResponse::Ok().json(ApiResponse::<()>::success_no_data(
                    "Logged out successfully"
                )));
            }
        }
    }
    
    Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
        "Invalid or missing authorization header"
    )))
}

// New endpoint to clean up expired sessions
pub async fn cleanup_sessions(pool: web::Data<PgPool>) -> Result<HttpResponse> {
    match sqlx::query("DELETE FROM user_sessions WHERE expires_at < CURRENT_TIMESTAMP")
        .execute(pool.get_ref())
        .await
    {
        Ok(result) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                "Sessions cleaned up successfully",
                format!("Deleted {} expired sessions", result.rows_affected())
            )))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to cleanup sessions"
            )))
        }
    }
}

pub fn auth_routes() -> Scope {
    web::scope("/auth")
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .route("/logout", web::post().to(logout))
        .route("/cleanup-sessions", web::post().to(cleanup_sessions))
} 