@echo off
echo 🚀 Starting Job Finder Backend...

echo 🔍 Checking for processes on port 8080...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :8080 ^| findstr LISTENING') do (
    echo 🔧 Killing process %%a on port 8080...
    taskkill /F /PID %%a >nul 2>&1
)

echo ✅ Port 8080 is now free!

echo 🔧 Configuring server to accept external connections...
set SERVER_HOST=0.0.0.0
set SERVER_PORT=8080
set DATABASE_URL=postgresql://jobuser:jobuser@localhost/connecting_opportunities
set JWT_SECRET=your-super-secret-jwt-key-change-this-in-production-please
set JWT_EXPIRATION=86400
set RUST_LOG=info

echo 🎯 Starting application...
echo 📡 Server will be accessible on all network interfaces (0.0.0.0:8080)

timeout /t 2 /nobreak >nul
cargo run 