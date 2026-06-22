// VS Code client for the fiqh language. Launches `fiqhc lsp` over stdio and wires it as a
// Language Server, so .fiqh files get live fiqh-cited squiggles (RIBA-1, GHARAR-1, …).
const { workspace } = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

function activate(context) {
  const cfg = workspace.getConfiguration("fiqh");
  const command = cfg.get("serverPath", "fiqhc");
  const args = ["lsp"];

  const serverOptions = {
    run: { command, args, transport: TransportKind.stdio },
    debug: { command, args, transport: TransportKind.stdio },
  };
  const clientOptions = {
    documentSelector: [{ scheme: "file", language: "fiqh" }],
    synchronize: { fileEvents: workspace.createFileSystemWatcher("**/*.fiqh") },
  };

  client = new LanguageClient("fiqh", "fiqh language server", serverOptions, clientOptions);
  client.start();
  context.subscriptions.push(client);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
