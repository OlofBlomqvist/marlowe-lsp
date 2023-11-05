$originalPath = $pwd
try {
    Set-Location .\Server
    cargo build --bin marlowe_lsp
} catch {
    write-error $_
} finally {
    Set-Location $originalPath
}