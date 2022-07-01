## MarloweLSP

An experimental LSP for Cardano Marlowe smart contracts.

### Requirements for building

* Cargo/Rust toolchain
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

* [NPM](https://www.npmjs.com/)

### How to build

If using PowerShell, you can run the build.ps1 script,
otherwise use "cargo build" in the server directory,
and "vsce package" in the client directory.

Don't forget to copy your binary to the ./client/server before generating the client package.






