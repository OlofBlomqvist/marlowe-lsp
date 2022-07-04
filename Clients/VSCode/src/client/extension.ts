import * as os from 'os';
const { exec } = require("child_process");
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
  
  let darwinbin = context.extensionPath + "/bin/marlowe_lsp_x86_64-apple-darwin.bin";
  exec("chmod +x \"" + darwinbin + "\"", (error, stdout, stderr) => {
    if (error) {
      LOGGAAAA.appendLine(`set chmod for darwin-bin error: ${error.message}`);
    }
  });

  let linbin = context.extensionPath + "/bin/marlowe_lsp_x86_64-unknown-linux-musl.bin";
  exec("chmod +x \"" + linbin + "\"", (error, stdout, stderr) => {
    if (error) {
      LOGGAAAA.appendLine(`set chmod for lin-bin error: ${error.message}`);
    }
  });

  let bin_path;
  let platform = os.platform();
  if(platform === "win32") {
    bin_path = context.extensionPath + "/bin/marlowe_lsp_x86_64-windows.exe"
  } else if (platform === "darwin") {
    bin_path = darwinbin
  } else if (platform === "linux") {
    bin_path = linbin
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
