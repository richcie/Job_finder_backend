# Database Setup

## Prerequisites
- PostgreSQL 12+ installed
- Database user with CREATE DATABASE privileges

## Setup Instructions

1. **Connect to PostgreSQL as superuser:**
   ```bash
   psql -U postgres
   ```

2. **Run the initialization script:**
   ```bash
   psql -U postgres -f db/init.sql
   ```

3. **Alternative: Create database manually:**
   ```sql
   CREATE DATABASE connecting_opportunities;
   CREATE USER job_finder_user WITH PASSWORD 'your_secure_password';
   GRANT ALL PRIVILEGES ON DATABASE connecting_opportunities TO job_finder_user;
   ```

4. **Run migrations:**
   ```bash
   psql -U job_finder_user -d connecting_opportunities -f db/init.sql
   ```

## Environment Variables
Set these in your `.env` file:
```
DATABASE_URL=postgresql://job_finder_user:your_secure_password@localhost/connecting_opportunities
JWT_SECRET=your_jwt_secret_key
RUST_LOG=info
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

## Database Schema

### Users Table
- Primary authentication table
- Supports multiple user roles: job_seeker, employer, admin
- Includes email verification and account status
- Optimized with indexes for common queries

### User Sessions Table
- JWT token management
- Session expiration tracking
- Cascade delete on user removal 