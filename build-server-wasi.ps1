$originalPath = $pwd
try {
    Set-Location .\Server
    cargo rustc --release --target wasm32-wasi-preview1-threads --bin marlowe_lsp -- -Clink-arg=--initial-memory=10485760 -Clink-arg=--max-memory=10485760
} catch {
    write-error $_
} finally {
    Set-Location $originalPath
}