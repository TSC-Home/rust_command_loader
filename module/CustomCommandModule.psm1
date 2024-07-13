# Importiere PSReadLine für verbesserte Befehlszeilenbearbeitung
Import-Module PSReadLine

# Konfiguriere PSReadLine Autovervollständigung
Set-PSReadLineOption -PredictionSource History

# Setze globale Variablen
$global:commandDir = "$env:USERPROFILE\commands"
$global:logDir = "$global:commandDir\logs"

# Funktion zum Initialisieren des Log-Verzeichnisses
function Initialize-LogDirectory {
    if (-not (Test-Path -Path $global:logDir)) {
        New-Item -ItemType Directory -Path $global:logDir -Force
    }
}

# Funktion zum Registrieren der benutzerdefinierten Befehle
function Register-CustomCommands {
    $errors = @()
    $commandFiles = Get-ChildItem -Path $global:commandDir -Filter *.exe

    if ($commandFiles.Count -eq 0) {
        Write-Host "No custom commands to register." -ForegroundColor Yellow
        return
    }

    $commandFiles | ForEach-Object {
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
            $logPath = "$global:logDir\$($error.Id).log"
            Set-Content -Path $logPath -Value "Error initializing command $($error.Name)"
            Write-Host "Custom command $($error.Name) has an error. Logs ID: $($error.Id)" -ForegroundColor Red
        }
    }
}

# Funktion zum Hinzufügen oder Bearbeiten von Befehlen
function cc {
    param (
        [Parameter(Mandatory=$false)]
        [string]$action,
        
        [Parameter(Mandatory=$false)]
        [string]$commandName,
        
        [Parameter(Mandatory=$false)]
        [string]$urlOrPath
    )

    $rustCommandLoader = "C:\Users\ZERO\Documents\GitHub\rust_command_loader\target\release\rust_command_loader.exe"

    if (-not $action) {
        Write-Host "No action specified. Use 'cc help' for usage information." -ForegroundColor Yellow
        return
    }

    switch ($action) {
        "add" {
            if ($urlOrPath) {
                & $rustCommandLoader add $commandName $urlOrPath
            } else {
                & $rustCommandLoader add $commandName
            }
        }
        "edit" {
            & $rustCommandLoader edit $commandName
        }
        "load" {
            & $rustCommandLoader load $commandName
            Register-CustomCommands
        }
        "reload" {
            if ($commandName -eq "all") {
                & $rustCommandLoader reload all
            } else {
                & $rustCommandLoader reload $commandName
            }
            Register-CustomCommands
        }
        "showlogs" {
            & $rustCommandLoader showlogs $commandName
        }
        "delete" {
            $confirmation = Read-Host "Are you sure you want to delete the command '$commandName'? (y/N)"
            if ($confirmation -eq "y") {
                & $rustCommandLoader delete $commandName -y
                if (Test-Path Function:\$commandName) {
                    Remove-Item Function:\$commandName -Force
                }
            } else {
                Write-Host "Command deletion cancelled." -ForegroundColor Yellow
            }
        }
        "help" {
            Write-Host "Usage: cc <action> [arguments]" -ForegroundColor Cyan
            Write-Host "Actions:" -ForegroundColor Cyan
            Write-Host "  add <command_name> [url_or_path] - Add a new command"
            Write-Host "  edit <command_name>              - Edit an existing command"
            Write-Host "  load <command_name>              - Load a specific command"
            Write-Host "  reload <command_name or 'all'>   - Reload a specific command or all commands"
            Write-Host "  showlogs <log_id or 0>           - Show logs (0 to open log directory)"
            Write-Host "  delete <command_name>            - Delete a command (requires confirmation)"
            Write-Host "  help                             - Show this help message"
        }
        default {
            Write-Host "Unknown action: $action" -ForegroundColor Red
            Write-Host "Use 'cc help' for usage information." -ForegroundColor Yellow
        }
    }
}

# Funktion zum Laden aller Befehle
function cload {
    cc reload all
}

# Export der Funktionen aus dem Modul
Export-ModuleMember -Function cc, cload

# Initialisiere das Log-Verzeichnis
Initialize-LogDirectory

Register-CustomCommands