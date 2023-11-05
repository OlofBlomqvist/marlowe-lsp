## WASI POC

This is a POC showing that it is possible to run Marlowe LSP as a WASI module inside of VSCODE, both
in desktop and web environments. It uses the preview version of wasi targets and preview versions of vscode-wasm extension.

There are a couple of stability issues with the preview vscode-wasm extension, and so the published vscode extension for MarloweLSP
currently uses WASM with custom worker communication implementation. 

### Compiling:

1. Build the LSP server with WASI script and place the resulting wasm file in the VSCode_WASI directory.
2. Build the client by running "npm run build" in the client directory.
3. Run the pack.sh script.
4. Install the resulting vsix file in vscode. 

### Known issues:

Large auto-complete notification messages can corrupt the buffer causing the extension host to stop working




