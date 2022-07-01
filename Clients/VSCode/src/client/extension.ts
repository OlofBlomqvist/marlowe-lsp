import * as os from 'os';
import { workspace, ExtensionContext } from "vscode";
import * as vs from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;
export function activate(context: ExtensionContext) {

  let LOGGAAAA = vs.window.createOutputChannel("MarloweLSP_Ext_Debug");
  LOGGAAAA.appendLine("Activating extension..")
   
  let bin_path;
  let platform = os.platform();
  if(platform === "win32") {
    bin_path = context.extensionPath + "/bin/marlowe_lsp_win_x86_64_.exe"
  } else if (platform === "darwin") {
    bin_path = context.extensionPath + "/bin/marlowe_lsp_x86_64-apple-darwin"
  } else if (platform === "linux") {
    bin_path = context.extensionPath + "/bin/marlowe_lsp_test4_x86_64-unknown-linux-musl"
  } else {
    throw "unsupported platform: " + platform
  }
  const serverOptions : ServerOptions = {
    transport: TransportKind.stdio,
    command: bin_path
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "Marlowe" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    },
  };

  LOGGAAAA.appendLine("Creating client!");
  client = new LanguageClient(
    "MarloweLSP",
    "MarloweLSP",
    serverOptions,
    clientOptions
  );
  client.traceOutputChannel.show();
  client.outputChannel.show();
  client.start();
  LOGGAAAA.appendLine("Started client.");

}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
