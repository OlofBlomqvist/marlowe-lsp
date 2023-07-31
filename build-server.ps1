$originalPath = $pwd
try {
    Set-Location .\Server
    wasm-pack build --target web --features "wasm"
    wasm-pack pack
} catch {
    write-error $_
} finally {
    Set-Location $originalPath
}