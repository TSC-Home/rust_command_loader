
$global:commandDir = "C:\Users\ZERO\commands"

function Register-CustomCommands {
    Get-ChildItem -Path $global:commandDir -Filter *.exe | ForEach-Object {
        $commandName = $_.BaseName
        $exePath = $_.FullName
        
        if (Test-Path Alias:\$commandName) {
            Remove-Item Alias:\$commandName -Force
        }
        if (Test-Path Function:\$commandName) {
            Remove-Item Function:\$commandName -Force
        }
        
        $functionDefinition = @"
function global:$commandName {
    & '$exePath' `$args
}
"@
        
        Invoke-Expression $functionDefinition

        if (Test-Path Function:\$commandName) {
            Write-Host "Registered command $commandName, executable: $exePath" -ForegroundColor Green
        } else {
            # Write-Host "Failed to register command $commandName, executable: $exePath" -ForegroundColor Red
        }
    }
}

function cedit {
    param (
        [string]$commandName
    )
    & "C:\Users\ZERO\Documents\GitHub\rust_command_loader\target\release\rust_command_loader.exe" cnc $commandName
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Command $commandName created/edited successfully." -ForegroundColor Green
    } else {
        # Write-Host "Failed to create/edit command $commandName." -ForegroundColor Red
    }
}

function cload {
    & "C:\Users\ZERO\Documents\GitHub\rust_command_loader\target\release\rust_command_loader.exe" cload
    if ($LASTEXITCODE -eq 0) {
        Register-CustomCommands
        # Write-Host "Commands loaded successfully." -ForegroundColor Green
    } else {
        # Write-Host "Failed to load some commands." -ForegroundColor Red
    }
}

function cdelete {
    param (
        [string]$commandName
    )
    $exePath = Join-Path $global:commandDir "$commandName.exe"
    $rsPath = Join-Path $global:commandDir "$commandName.rs"

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


Export-ModuleMember -Function cedit, cload, cdelete, chelp