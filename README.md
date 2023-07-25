## MarloweLSP

An experimental LSP for Cardano Marlowe using [marlowe-rs](https://github.com/OlofBlomqvist/marlowe_rust).

For the VSCode extension, go [here](https://marketplace.visualstudio.com/items?itemName=OlofBlomqvist.marlowelsp).

The VSCode extension is soon going to be changed to use wasm, so that it can be used in vscode.dev, and in the monaco editor. There is a marlowe-lsp-wasm-test extension published as a poc for this work.

https://user-images.githubusercontent.com/5273471/177008205-3f66a2d5-2082-4f6f-b6b8-e7ef8ef5a5b3.mp4

### Requirements for building

* Cargo/Rust nightly toolchain
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

* [NPM](https://www.npmjs.com/)

### How to build

If using PowerShell, you can run the build.ps1 script,
otherwise use "cargo build" in the server directory,
and "vsce package" in the client directory.

Don't forget to copy your binary to the ./client/bin directory before generating the client package.






