[CmdletBinding()]
param()

Remove-Item .\Clients\VSCode\*.vsix -ErrorAction SilentlyContinue
Remove-Item .\Clients\VSCode\build\client\extension.js -verbose:$VerbosePreference  -ErrorAction SilentlyContinue
Set-Location ./Clients/VSCode -ErrorAction Stop
vsce package
if($LASTEXITCODE -ne 0) {
    Write-Error "Failed to build client."
}
Set-Location "../"
Set-Location "../"
