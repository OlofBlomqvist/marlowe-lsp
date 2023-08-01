$originalPath = $pwd
try {
    Set-Location .\Server
    wasm-pack build --target web
    wasm-pack pack
} catch {
    write-error $_
} finally {
    Set-Location $originalPath
}