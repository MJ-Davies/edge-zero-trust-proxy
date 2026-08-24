Write-Host "===================================================="
Write-Host "Starting Edge Zero Trust Proxy Environment..."
Write-Host "===================================================="

Write-Host "[Mock Backend] Spawning minimized window..."
$backendArgs = "-NoProfile -Command `"cd mock-backend; npx wrangler dev --port 8788`""
$backendProcess = Start-Process powershell -ArgumentList $backendArgs -WindowStyle Minimized -PassThru
Write-Host "Mock Backend Root PID: $($backendProcess.Id)"

Start-Sleep -Seconds 2

Write-Host "[Rust Proxy] Spawning minimized window..."
$proxyArgs = "-NoProfile -Command `"cd edge-zero-trust-proxy; npx wrangler dev`""
$proxyProcess = Start-Process powershell -ArgumentList $proxyArgs -WindowStyle Minimized -PassThru
Write-Host "Rust Proxy Root PID: $($proxyProcess.Id)"

Write-Host "`n===================================================="
Write-Host "Both services are running minimized."
Write-Host "Type 'EXIT' and press Enter to kill all processes."
Write-Host "===================================================="

while ($true) {
    $userInput = Read-Host "Enter command"
    if ($userInput -eq "EXIT") {
        break
    }
}

Write-Host "`nShutting down processes..."

function Kill-ProcessTree {
    param ([int]$parentId)
    $children = Get-CimInstance Win32_Process -Filter "ParentProcessId = $parentId" -ErrorAction SilentlyContinue
    foreach ($child in $children) {
        Kill-ProcessTree -parentId $child.ProcessId
    }
    Stop-Process -Id $parentId -Force -ErrorAction SilentlyContinue
}

if ($null -ne $backendProcess -and -not $backendProcess.HasExited) {
    Write-Host "Terminating Mock Backend process tree (PID: $($backendProcess.Id))..."
    Kill-ProcessTree -parentId $backendProcess.Id
}

if ($null -ne $proxyProcess -and -not $proxyProcess.HasExited) {
    Write-Host "Terminating Rust Proxy process tree (PID: $($proxyProcess.Id))..."
    Kill-ProcessTree -parentId $proxyProcess.Id
}

Write-Host "All services stopped cleanly."