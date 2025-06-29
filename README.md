# Job Finder Backend

A high-performance Rust backend for a job finder application built with Actix-web and PostgreSQL.

## Features

- **High Performance**: Optimized Actix-web configuration with connection pooling
- **JWT Authentication**: Secure user authentication with JWT tokens
- **PostgreSQL Integration**: Async database operations with connection pooling
- **Input Validation**: Comprehensive request validation using the validator crate
- **CORS Support**: Configurable CORS for frontend integration
- **Health Checks**: Built-in health check endpoints
- **Role-based Access**: Support for different user roles (job_seeker, employer, admin)

## Quick Start

### Prerequisites

- Rust 1.70+ 
- PostgreSQL 12+
- Git

### Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd Job_finder_backend
   ```

2. **Set up PostgreSQL database**
   ```bash
   # Connect to PostgreSQL
   psql -U postgres
   
   # Run the database setup
   psql -U postgres -f db/init.sql
   ```

3. **Configure environment variables**
   Copy the environment template and configure it:
   ```bash
   cp environment.config.example .env
   ```
   Then edit the `.env` file with your configuration:
   ```env
   DATABASE_URL=postgresql://postgres:password@localhost/job_finder_db
   JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
   JWT_EXPIRATION=86400
   SERVER_HOST=127.0.0.1
   SERVER_PORT=8080
   RUST_LOG=info
   ```
   
   **Note**: For production, make sure to use a strong JWT secret (at least 32 characters). You can generate one using:
   ```bash
   openssl rand -base64 32
   ```

4. **Install dependencies and run**
   ```bash
   cargo run
   ```

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register a new user
- `POST /api/v1/auth/login` - Login user

### Users
- `GET /api/v1/users/profile` - Get current user profile (requires auth)
- `PUT /api/v1/users/profile` - Update user profile (requires auth)
- `GET /api/v1/users/{id}` - Get user by ID (requires auth)

### Health
- `GET /` - API status
- `GET /api/v1/health` - Health check with database connection test

## Database Schema

### Users Table
```sql
- id: SERIAL PRIMARY KEY
- username: VARCHAR(50) UNIQUE NOT NULL
- email: VARCHAR(100) UNIQUE NOT NULL
- password_hash: VARCHAR(255) NOT NULL
- first_name: VARCHAR(50)
- last_name: VARCHAR(50)
- phone: VARCHAR(20)
- role: VARCHAR(20) DEFAULT 'job_seeker'
- is_active: BOOLEAN DEFAULT true
- email_verified: BOOLEAN DEFAULT false
- created_at: TIMESTAMP WITH TIME ZONE
- updated_at: TIMESTAMP WITH TIME ZONE
```

## Performance Optimizations

- **Connection Pooling**: Optimized PostgreSQL connection pool (5-20 connections)
- **Multi-threading**: Uses all available CPU cores
- **High Connection Limits**: Supports up to 25,000 concurrent connections
- **Database Indexing**: Strategic indexes on frequently queried columns
- **JWT Middleware**: Efficient token validation with minimal overhead

## Development

### Running Tests
```bash
cargo test
```

### Building for Production
```bash
cargo build --release
```

### Database Migrations
Database migrations are handled automatically on startup using SQLx migrate.

## Security Features

- **Password Hashing**: bcrypt for secure password storage
- **JWT Tokens**: Stateless authentication with configurable expiration
- **Input Validation**: Comprehensive validation for all user inputs
- **CORS Configuration**: Configurable CORS settings for frontend integration
- **SQL Injection Prevention**: Parameterized queries with SQLx

## Configuration

The application can be configured through environment variables:

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secret key for JWT token signing
- `JWT_EXPIRATION`: Token expiration time in seconds
- `SERVER_HOST`: Server bind address
- `SERVER_PORT`: Server port
- `RUST_LOG`: Logging level

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License. 