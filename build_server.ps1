[CmdletBinding()]
param()

Write-Output "Preparing for build"
Remove-Item .\Server\target\release -Recurse -verbose:$VerbosePreference  -ErrorAction SilentlyContinue
Set-Location .\Server -verbose:$VerbosePreference -ErrorAction Stop
Write-Output "Building server"

cargo build --release

if($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build server!" -ErrorAction stop
}

Set-Location ".."