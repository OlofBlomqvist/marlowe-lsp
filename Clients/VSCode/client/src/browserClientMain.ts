import { ExtensionContext, Uri ,commands } from 'vscode';
import { LanguageClientOptions } from 'vscode-languageclient';

import { LanguageClient } from 'vscode-languageclient/browser';

import * as vscode from "vscode";

export function activate(context: ExtensionContext) {

	const documentSelector = [{ language: 'marlowe' }];

	const clientOptions: LanguageClientOptions = {
		documentSelector,
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher("**/.clientrc"),
		},
	};

	const x = createWorkerLanguageClient(context, clientOptions);
	x.w.onerror = console.error;
	x.w.onmessageerror = console.error;
	x.c.onDidChangeState(x => {
		console.log("STATE CHANGED FROM " + x.oldState + " to " + x.newState);
	});

}

function createWorkerLanguageClient(context: ExtensionContext, clientOptions: LanguageClientOptions) {
	
	const serverMain = Uri.joinPath(context.extensionUri, 'dist/browserServerMain.js');
	const worker = new Worker(serverMain.toString(true));
	
	worker.onmessage = e => {
		if(e.data=="getPath") {
			worker.postMessage("setPath§"+context.extensionUri.toString());
		}
		if(e.data=="ready!") {
			console.log("STARTING LSP CLIENT");
			client.start();
		}
	};
	
	const client = new LanguageClient('marlowelsp', 'marlowelsp', clientOptions, worker);	
	
	//client.traceOutputChannel.show();
	//client.outputChannel.show();
	return {
		c : client,
		w : worker
	};
}
