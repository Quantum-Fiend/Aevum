$ErrorActionPreference = "Stop"

Write-Host "Starting Aevum Time-Travel Debugger Environment..." -ForegroundColor Cyan

# Check if Go is installed
if (!(Get-Command go -ErrorAction SilentlyContinue)) {
    Write-Error "Go is not installed or not in PATH."
    exit 1
}

# Check if Node is installed
if (!(Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Error "Node.js/npm is not installed or not in PATH."
    exit 1
}

# Define paths
$root = Get-Location
$coordinatorDir = Join-Path $root "coordinator"
$uiDir = Join-Path $root "ui"

# Function to stop background jobs on exit
function Stop-Jobs {
    Write-Host "Stopping services..." -ForegroundColor Yellow
    Get-Job | Stop-Job
    Get-Job | Remove-Job
}
Register-EngineEvent -SourceIdentifier PowerShell.Exiting -SupportEvent -Action { Stop-Jobs }

# Start Coordinator
Write-Host "Starting Coordinator..." -ForegroundColor Green
$coordinatorJob = Start-Job -ScriptBlock {
    param($dir)
    Set-Location $dir
    go run main.go
} -ArgumentList $coordinatorDir

# Start UI
Write-Host "Starting UI..." -ForegroundColor Green
$uiJob = Start-Job -ScriptBlock {
    param($dir)
    Set-Location $dir
    npm run dev
} -ArgumentList $uiDir

Write-Host "Services started!" -ForegroundColor Cyan
Write-Host "Coordinator running on port 9876"
Write-Host "UI running on http://localhost:5173"
Write-Host "Press Ctrl+C to stop..."

# Loop to keep script running and stream output
try {
    while ($true) {
        Receive-Job -Job $coordinatorJob -Keep | ForEach-Object { Write-Host "[Coordinator] $_" }
        Receive-Job -Job $uiJob -Keep | ForEach-Object { Write-Host "[UI] $_" }
        Start-Sleep -Seconds 1
        
        if ($coordinatorJob.State -ne 'Running' -or $uiJob.State -ne 'Running') {
            Write-Error "One of the services crashed."
            break
        }
    }
}
finally {
    Stop-Jobs
}
