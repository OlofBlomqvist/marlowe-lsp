$originalPath = $pwd
try {
    copy-item .\Server\pkg\marlowe_lsp_lib_bg.wasm
    Set-Location .\Clients\VSCode
    ./build-client.ps1
} catch {
    write-error $_
} finally {
    Set-Location $originalPath
}