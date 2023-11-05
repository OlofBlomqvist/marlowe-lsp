/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import { ExtensionContext, Uri, window, workspace } from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
import { Wasm, ProcessOptions, Stdio } from '@vscode/wasm-wasi';
import { runServerProcess } from './lspServer';


let client: LanguageClient;
const channel = window.createOutputChannel('HAXERY');

export async function activate(context: ExtensionContext) {
	const wasm: Wasm = await Wasm.load();

	const serverOptions: ServerOptions = async () => {
		const stdio: Stdio = {
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

		const options: ProcessOptions = {
			stdio: stdio,
			encoding: 'utf-8',
			mountPoints: [
				{ kind: 'workspaceFolder' },
			]
		};
		const filename = Uri.joinPath(context.extensionUri, 'marlowe_lsp.wasm');
		const bits = await workspace.fs.readFile(filename);
		const module = await WebAssembly.compile(bits);
		const process = await wasm.createProcess('lsp-server', module, { initial: 160, maximum: 160, shared: true }, options);

		const decoder = new TextDecoder('utf-8');
		
		let expected_next_length = 0;

		process.stdout!.onData((data) => {

			let dataaa = decoder.decode(data).trim();

			console.log("got raw message: '",dataaa+"'")

			if(dataaa.startsWith("Content")) {
				console.log("received content length for next message: ", dataaa)
				expected_next_length = Number.parseInt(dataaa.split(" ")[1].trim())
			}

			if(!dataaa.startsWith("Content")) {
				
				if(expected_next_length != dataaa.length) {
					console.warn("hmmm expected length of " + expected_next_length + " but see: " + dataaa.length)
				}
				try {
					JSON.parse(dataaa)
				} catch {
					console.error("invalid json!!: ", dataaa)
				}

			}
		});
		process.stderr!.onData((data) => {
			channel.append(decoder.decode(data).trim());
		});

		return runServerProcess(process);
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [
			{ language: 'marlowe' }
		],
		outputChannel: channel,
		diagnosticCollectionName: 'markers',
	};

	client = new LanguageClient('lspClient', 'LSP Client', serverOptions, clientOptions);
	try {
		await client.start();
	} catch (error) {
		client.error(`Start failed`, error, 'force');
	}
}

export function deactivate() {
	return client.stop();
}