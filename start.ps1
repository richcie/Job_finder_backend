# Job Finder Backend Starter Script
# This script kills any process using port 8080 and starts the application

Write-Host "Starting Job Finder Backend..." -ForegroundColor Green

# Function to kill process on specific port
function Kill-ProcessOnPort {
    param([int]$Port)
    
    Write-Host "Checking for processes on port $Port..." -ForegroundColor Yellow
    
    try {
        # Find process using the port
        $process = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue | Select-Object OwningProcess -Unique
        
        if ($process) {
            foreach ($proc in $process) {
                if ($proc.OwningProcess -gt 0) {
                    Write-Host "Killing process $($proc.OwningProcess) on port $Port..." -ForegroundColor Red
                    Stop-Process -Id $proc.OwningProcess -Force -ErrorAction SilentlyContinue
                }
            }
            Write-Host "Port $Port is now free!" -ForegroundColor Green
        } else {
            Write-Host "Port $Port is already free!" -ForegroundColor Green
        }
    }
    catch {
        Write-Host "Could not check port $Port, continuing anyway..." -ForegroundColor Yellow
    }
}

# Kill any process on port 8080
Kill-ProcessOnPort -Port 8080

# Wait a moment for processes to fully terminate
Start-Sleep -Seconds 2

# Set environment variables for external access
Write-Host "Configuring server to accept external connections..." -ForegroundColor Cyan
$env:SERVER_HOST = "0.0.0.0"
$env:SERVER_PORT = "8080"
$env:DATABASE_URL = "postgresql://jobuser:jobuser@localhost/connecting_opportunities"
$env:JWT_SECRET = "your-super-secret-jwt-key-change-this-in-production-please"
$env:JWT_EXPIRATION = "86400"
$env:RUST_LOG = "info"

# Start the application
Write-Host "Starting application with cargo run..." -ForegroundColor Cyan
Write-Host "Server will be accessible on all network interfaces (0.0.0.0:8080)" -ForegroundColor Green
cargo run 