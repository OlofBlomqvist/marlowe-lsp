"use strict";
/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
const wasm_wasi_1 = require("@vscode/wasm-wasi");
const lspServer_1 = require("./lspServer");
let client;
const channel = vscode_1.window.createOutputChannel('HAXERY');
async function activate(context) {
    const wasm = await wasm_wasi_1.Wasm.load();
    const serverOptions = async () => {
        const stdio = {
            in: {
                kind: 'pipeIn',
            },
            out: {
                kind: 'pipeOut'
            },
            err: {
                kind: 'pipeOut'
            }
        };
        const options = {
            stdio: stdio,
            encoding: 'utf-8',
            mountPoints: [
                { kind: 'workspaceFolder' },
            ]
        };
        const filename = vscode_1.Uri.joinPath(context.extensionUri, 'marlowe_lsp.wasm');
        const bits = await vscode_1.workspace.fs.readFile(filename);
        const module = await WebAssembly.compile(bits);
        const process = await wasm.createProcess('lsp-server', module, { initial: 160, maximum: 160, shared: true }, options);
        const decoder = new TextDecoder('utf-8');
        let expected_next_length = 0;
        process.stdout.onData((data) => {
            let dataaa = decoder.decode(data).trim();
            console.log("got raw message: '", dataaa + "'");
            if (dataaa.startsWith("Content")) {
                console.log("received content length for next message: ", dataaa);
                expected_next_length = Number.parseInt(dataaa.split(" ")[1].trim());
            }
            if (!dataaa.startsWith("Content")) {
                if (expected_next_length != dataaa.length) {
                    console.warn("hmmm expected length of " + expected_next_length + " but see: " + dataaa.length);
                }
                try {
                    JSON.parse(dataaa);
                }
                catch {
                    console.error("invalid json!!: ", dataaa);
                }
            }
        });
        process.stderr.onData((data) => {
            channel.append(decoder.decode(data).trim());
        });
        return (0, lspServer_1.runServerProcess)(process);
    };
    const clientOptions = {
        documentSelector: [
            { language: 'marlowe' }
        ],
        outputChannel: channel,
        diagnosticCollectionName: 'markers',
    };
    client = new node_1.LanguageClient('lspClient', 'LSP Client', serverOptions, clientOptions);
    try {
        await client.start();
    }
    catch (error) {
        client.error(`Start failed`, error, 'force');
    }
}
exports.activate = activate;
function deactivate() {
    return client.stop();
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map