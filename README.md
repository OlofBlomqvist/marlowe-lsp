## MarloweLSP

An experimental LSP for Cardano Marlowe using [marlowe_rust](https://github.com/OlofBlomqvist/marlowe_rust).

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






