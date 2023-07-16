## MarloweLSP

An experimental LSP for Cardano Marlowe using [marlowe-rs](https://github.com/OlofBlomqvist/marlowe_rust).

For the VSCode extension, go [here](https://marketplace.visualstudio.com/items?itemName=OlofBlomqvist.marlowelsp).

Note for vscode extension: Marlowe playground has not yet (22-10-03) updated to use Address instead of PK and so if you want to view contracts
created from there, or any old contract that uses PK, you need to use version 0.11.

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






