$global:commandDir = [System.IO.Path]::Combine($env:USERPROFILE, "commands")
$global:logDir = [System.IO.Path]::Combine($global:commandDir, "logs")

function Register-CustomCommands {
    $errors = @()
    Get-ChildItem -Path $global:commandDir -Filter *.exe | ForEach-Object {
        $commandName = $_.BaseName
        $exePath = $_.FullName

        if (Test-Path Function:\$commandName) {
            Remove-Item Function:\$commandName -Force
        }

        $functionDefinition = @"
function global:$commandName {
    & '$exePath' `$args
}
"@
        Invoke-Expression $functionDefinition

        if (-not (Test-Path Function:\$commandName)) {
            $errorId = [System.Guid]::NewGuid().ToString()
            $errors += @{ Name = $commandName; Id = $errorId }
        }
    }

    if ($errors.Count -eq 0) {
        Write-Host "All commands initialized successfully." -ForegroundColor Green
    } else {
        foreach ($error in $errors) {
            $logPath = [System.IO.Path]::Combine($global:logDir, "$($error.Id).log")
            Set-Content -Path $logPath -Value "Error initializing command $($error.Name)"
            Write-Host "Custom command $($error.Name) has an error. Logs ID: $($error.Id)" -ForegroundColor Red
        }
    }
}

function cedit {
    param (
        [string]$commandName
    )
    & [System.IO.Path]::Combine($env:USERPROFILE, "commands\rust_command_loader.exe") cnc $commandName
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Command $commandName created/edited successfully." -ForegroundColor Green
    } else {
        Write-Host "Failed to create/edit command $commandName." -ForegroundColor Red
    }
}

function cload {
    & [System.IO.Path]::Combine($env:USERPROFILE, "commands\rust_command_loader.exe") cload
    if ($LASTEXITCODE -eq 0) {
        Register-CustomCommands
        Write-Host "Commands loaded successfully." -ForegroundColor Green
    } else {
        Write-Host "Failed to load some commands." -ForegroundColor Red
    }
}

function cdelete {
    param (
        [string]$commandName
    )
    $exePath = [System.IO.Path]::Combine($global:commandDir, "$commandName.exe")
    $rsPath = [System.IO.Path]::Combine($global:commandDir, "$commandName.rs")

    if (Test-Path $exePath) {
        Remove-Item $exePath -Force
        Write-Host "Deleted executable: $exePath" -ForegroundColor Green
    } else {
        Write-Host "Executable not found: $exePath" -ForegroundColor Yellow
    }

    if (Test-Path $rsPath) {
        Remove-Item $rsPath -Force
        Write-Host "Deleted source file: $rsPath" -ForegroundColor Green
    } else {
        Write-Host "Source file not found: $rsPath" -ForegroundColor Yellow
    }

    if (Test-Path Function:\$commandName) {
        Remove-Item Function:\$commandName -Force
        Write-Host "Removed function: $commandName" -ForegroundColor Green
    }

    Write-Host "Command $commandName has been deleted." -ForegroundColor Green
}

function chelp {
    $commands = Get-ChildItem -Path $global:commandDir -Filter *.exe | Select-Object -ExpandProperty BaseName

    Write-Host "Available custom commands:" -ForegroundColor Cyan
    foreach ($command in $commands) {
        Write-Host "  $command" -ForegroundColor Yellow
    }

    Write-Host "`nUsage:" -ForegroundColor Cyan
    Write-Host "  cedit <command_name>    : Create or edit a command" -ForegroundColor Yellow
    Write-Host "  cload                 : Load all commands" -ForegroundColor Yellow
    Write-Host "  cdelete <command_name>: Delete a command" -ForegroundColor Yellow
    Write-Host "  chelp                 : Display this help message" -ForegroundColor Yellow

    Write-Host "`nTo use a command, simply type its name." -ForegroundColor Cyan
}

function showlog {
    param (
        [string]$logId
    )
    $logPath = [System.IO.Path]::Combine($global:logDir, "$logId.log")
    if (Test-Path $logPath) {
        Get-Content $logPath
    } else {
        Write-Host "Log file not found for ID: $logId" -ForegroundColor Yellow
    }
}

Export-ModuleMember -Function cedit, cload, cdelete, chelp, showlog

# Register commands on module import
Register-CustomCommands
