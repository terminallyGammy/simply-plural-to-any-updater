
$ErrorActionPreference = "Stop"



# Adapted from https://github.com/stopthatastronaut/poshdotenv/blob/master/module/dotenv/functions/Push-DotEnv.ps1
function Read-DotEnv {
    [CmdletBinding(SupportsShouldProcess, DefaultParameterSetName = 'Name')]
    [OutputType([Hashtable])]
    param(
        [Parameter(Mandatory, ParameterSetName = 'Path', Position = 0)]
        [string]$Path,
        [Parameter(ParameterSetName = 'Name', Position = 0)]
        [string]$Name = '.env',
        [Parameter(ParameterSetName = 'Name', Position = 1)]
        [string]$Environment,
        [Parameter(ParameterSetName = 'Name')]
        [switch]$Up,
        [switch]$AllowClobber
    )

    if ($PSCmdlet.ParameterSetName -eq 'Name') {
        $pattern = "(^$([regex]::Escape($Name))$)"
        if ($Environment) { $pattern += "|(^$([regex]::Escape("$Name.$Environment"))$)" }

        $searchDir = Get-Item (Get-Location)
        do {
            Write-Verbose "looking in $($searchDir.FullName)..."
            $envfiles = @(Get-ChildItem $searchDir.FullName -File | Where-Object { $_.Name -match $pattern }) | Sort-Object
            $searchDir = $searchDir.Parent
        } while ($envfiles.Count -eq 0 -and $searchDir -and $Up)
        "Found $($envfiles.Count) .env files:" | Write-Verbose
        $envfiles | Write-Verbose
    }
    else {
        $envfiles = Resolve-Path $Path
    }

    if (-not $envfiles) { return }

    $newEnv = @{}
    foreach ($file in $envfiles) {
        Write-Debug "processing file: $file"

        foreach ($line in $file | Get-Content) {
            $line = $line.Trim()

            if ($line -eq '' -or $line -like '#*') {
                continue
            }

            $key, $value = ($line -split '=', 2).Trim()

            if ($value -like '"*"') {
                # expand \n to `n for double quoted values
                $value = $value -replace '^"|"$', '' -replace '(?<!\\)(\\n)', "`n"
            }
            elseif ($value -like "'*'") {
                $value = $value -replace "^'|'$", ''
            }

            $newEnv[$key] = $value
        }
    }

    foreach ($item in $newEnv.GetEnumerator()) {
        if ( -not (Test-Path "Env:\$($item.Name)") -or $AllowClobber ) {
            if ($PSCmdlet.ShouldProcess("`$env:$($item.Name)", "Set value to '$($item.Value)'")) {
                [System.Environment]::SetEnvironmentVariable($item.Name, $item.Value)
            }
        }
    }
}


########################### MAIN ###############################


# TODO. continue here.

# 1. Load defaults.env
# 2. Set SERVE_API=false
# 3. If local sps-vrcupdater.env exists, then load it.
# 3.a Otherwise create it and ask the user to edit it. Then exit.
# 4. Download executable from github.
# 5. Execute it with the loaded environment variables.

Read-DotEnv -Path defaults.env
Read-DotEnv -Path dev.pwsh.vrcupdater.env

[System.Environment]::GetEnvironmentVariable("SPS_API_BASE_URL")

$username = [System.Environment]::GetEnvironmentVariable("VRCHAT_USERNAME")

Write-Output "$username"
