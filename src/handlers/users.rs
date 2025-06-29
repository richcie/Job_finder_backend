use actix_web::{web, HttpResponse, Result, Scope, HttpRequest, HttpMessage};
use sqlx::{PgPool, Row};

use crate::models::{ApiResponse, Claims, User, UserPublic, UpdateUserRequest};
use crate::cache::CacheManager;
use crate::database;

pub async fn get_profile(req: HttpRequest) -> Result<HttpResponse> {
    // Extract claims from request extensions (set by auth middleware)
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>().unwrap();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        "Profile retrieved successfully",
        claims
    )))
}

pub async fn get_user_by_id(
    pool: web::Data<PgPool>,
    cache: web::Data<CacheManager>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    // Try cache first for better performance
    if let Some(cached_user) = cache.get_user(user_id).await {
        return Ok(HttpResponse::Ok().json(ApiResponse::success(
            "User found (cached)",
            cached_user
        )));
    }
    
    // If not in cache, query database with optimized query
    let user_result = database::get_user_by_id_optimized(&pool, user_id).await;

    match user_result {
        Ok(Some(row)) => {
            let user_public = UserPublic {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                phone: row.get("phone"),
                role: row.get("role"),
                professional_role: row.get("professional_role"),
                company_name: row.get("company_name"),
                is_active: row.get("is_active"),
                email_verified: row.get("email_verified"),
                created_at: row.get("created_at"),
            };
            
            // Cache the result for future requests
            cache.cache_user(user_public.clone()).await;
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                "User found",
                user_public
            )))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "User not found"
            )))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error"
            )))
        }
    }
}

pub async fn update_profile(
    pool: web::Data<PgPool>,
    cache: web::Data<CacheManager>,
    req: HttpRequest,
    update_data: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    // Extract claims from request extensions
    let user_id: i32 = {
        let extensions = req.extensions();
        let claims = extensions.get::<Claims>().unwrap();
        claims.sub.parse().unwrap()
    };

    // Input validation
    if let Some(username) = &update_data.username {
        if username.trim().is_empty() || username.len() < 3 {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Username must be at least 3 characters long"
            )));
        }
    }
    
    if let Some(email) = &update_data.email {
        if email.trim().is_empty() || !email.contains('@') {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Valid email is required"
            )));
        }
    }

    // Build dynamic query based on provided fields
    let mut query_parts = vec!["UPDATE users SET updated_at = CURRENT_TIMESTAMP".to_string()];
    let mut params: Vec<String> = vec![];
    let mut param_count = 1;

    if let Some(username) = &update_data.username {
        query_parts.push(format!("username = ${param_count}"));
        params.push(username.clone());
        param_count += 1;
    }

    if let Some(email) = &update_data.email {
        query_parts.push(format!("email = ${param_count}"));
        params.push(email.clone());
        param_count += 1;
    }

    if let Some(first_name) = &update_data.first_name {
        query_parts.push(format!("first_name = ${param_count}"));
        params.push(first_name.clone());
        param_count += 1;
    }

    if let Some(last_name) = &update_data.last_name {
        query_parts.push(format!("last_name = ${param_count}"));
        params.push(last_name.clone());
        param_count += 1;
    }

    if let Some(phone) = &update_data.phone {
        query_parts.push(format!("phone = ${param_count}"));
        params.push(phone.clone());
        param_count += 1;
    }

    if let Some(professional_role) = &update_data.professional_role {
        query_parts.push(format!("professional_role = ${param_count}"));
        params.push(professional_role.clone());
        param_count += 1;
    }

    if let Some(company_name) = &update_data.company_name {
        query_parts.push(format!("company_name = ${param_count}"));
        params.push(company_name.clone());
        param_count += 1;
    }

    if params.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "No fields to update"
        )));
    }

    let query = format!(
        "{} WHERE id = ${param_count} RETURNING id, username, email, password_hash, first_name, last_name, phone, role, professional_role, company_name, is_active, email_verified, created_at, updated_at",
        query_parts.join(", ")
    );

    // Create query with parameters
    let mut sqlx_query = sqlx::query_as::<_, User>(&query);
    for param in &params {
        sqlx_query = sqlx_query.bind(param);
    }
    sqlx_query = sqlx_query.bind(user_id);

    let user_result = sqlx_query.fetch_optional(pool.get_ref()).await;

    match user_result {
        Ok(Some(user)) => {
            let user_public: UserPublic = user.into();
            
            // Update cache with new data
            cache.cache_user(user_public.clone()).await;
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                "Profile updated successfully",
                user_public
            )))
        }
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "User not found"
            )))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.constraint().is_some() => {
            Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                "Username or email already exists"
            )))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to update profile"
            )))
        }
    }
}

// New endpoint for batch user retrieval
pub async fn get_users_batch(
    pool: web::Data<PgPool>,
    cache: web::Data<CacheManager>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let empty_string = String::new();
    let ids_param = query.get("ids").unwrap_or(&empty_string);
    let user_ids: Result<Vec<i32>, _> = ids_param
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().parse::<i32>())
        .collect();

    let user_ids = match user_ids {
        Ok(ids) if !ids.is_empty() && ids.len() <= 100 => ids, // Limit to 100 users per request
        Ok(_) => return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Invalid user IDs or too many requested (max 100)"
        ))),
        Err(_) => return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Invalid user ID format"
        ))),
    };

    // Try to get users from cache first
    let mut cached_users = Vec::new();
    let mut missing_ids = Vec::new();

    for id in &user_ids {
        if let Some(user) = cache.get_user(*id).await {
            cached_users.push(user);
        } else {
            missing_ids.push(*id);
        }
    }

    // Fetch missing users from database
    if !missing_ids.is_empty() {
        match database::get_multiple_users(&pool, &missing_ids).await {
            Ok(rows) => {
                for row in rows {
                    let user_public = UserPublic {
                        id: row.get("id"),
                        username: row.get("username"),
                        email: row.get("email"),
                        first_name: row.get("first_name"),
                        last_name: row.get("last_name"),
                        phone: row.get("phone"),
                        role: row.get("role"),
                        professional_role: row.get("professional_role"),
                        company_name: row.get("company_name"),
                        is_active: row.get("is_active"),
                        email_verified: row.get("email_verified"),
                        created_at: row.get("created_at"),
                    };
                    
                    // Cache the user
                    cache.cache_user(user_public.clone()).await;
                    cached_users.push(user_public);
                }
            }
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Database error"
                )));
            }
        }
    }

    // Sort users by ID to maintain consistent ordering
    cached_users.sort_by_key(|u| u.id);

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        "Users retrieved successfully",
        cached_users
    )))
}

pub fn user_routes() -> Scope {
    web::scope("/users")
        .route("/profile", web::get().to(get_profile))
        .route("/profile", web::put().to(update_profile))
        .route("/batch", web::get().to(get_users_batch))
        .route("/{id}", web::get().to(get_user_by_id))
} 