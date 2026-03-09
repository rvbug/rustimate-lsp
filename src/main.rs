use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use dashmap::DashMap;

mod completion;
mod diagnostics;
mod parser;

use completion::completions;
use diagnostics::collect_diagnostics;
use parser::RustimateParser;

#[derive(Debug)]
struct Backend {
    #[allow(dead_code)]
    client: Client,
    documents: DashMap<String, String>, 
}

fn get_line_context(text: &str, line: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line >= lines.len() { return "".to_string(); }
    lines[line].to_string() // Keep whitespace for "scene " check
}


#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![":".to_string(), "{".to_string(), " ".to_string(), "\"".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> { Ok(()) }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.insert(params.text_document.uri.to_string(), params.text_document.text);
    }

    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
    if let Some(change) = params.content_changes.into_iter().next() {

        // store document
        self.documents.insert(
            params.text_document.uri.to_string(),
            change.text.clone(),
        );

        let mut parser = RustimateParser::new();

        if let Some(tree) = parser.parse(&change.text) {

            let diagnostics = collect_diagnostics(&tree, &change.text);

            self.client
                .publish_diagnostics(
                    params.text_document.uri.clone(),
                    diagnostics,
                    None,
                )
                .await;
        }
    }
}



    async fn did_save(&self, params: DidSaveTextDocumentParams) {

        let uri = params.text_document.uri.to_string();
        let text = match self.documents.get(&uri) {
            Some(t) => t.value().clone(),
            None => return,
        };

        let mut parser = RustimateParser::new();

        if let Some(tree) = parser.parse(&text) {
            let diagnostics = collect_diagnostics(&tree, &text);

            self.client
                .publish_diagnostics(
                    params.text_document.uri,
                    diagnostics,
                    None,
                )
            .await;
        }
    }
   

  
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {

        let uri = params.text_document_position.text_document.uri.to_string();
        let pos = params.text_document_position.position;

        let text = match self.documents.get(&uri) {
            Some(t) => t.value().clone(),
            None => return Ok(None),
        };

        let line = text
            .lines()
            .nth(pos.line as usize)
            .unwrap_or("")
            .trim();

        if line.is_empty() {
            return Ok(None);
        }
        let mut parser = RustimateParser::new();

        if let Some(tree) = parser.parse(&text) {

            if let Some(node) = parser::node_at_position(
                &tree,
                pos.line as usize,
                pos.character as usize,
            ) {

                let context = parser::find_block_context(node);
                let mode = parser::detect_scene_mode(node, &text);

                let items = completions(context, mode, line);

                return Ok(Some(CompletionResponse::Array(items)));
            }
        }

        Ok(None)
    }


   
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;
        let doc_text = match self.documents.get(&uri) {
            Some(t) => t.value().clone(),
            None => return Ok(None),
        };

        let current_line = get_line_context(&doc_text, position.line as usize);

        let content = if current_line.contains("mode: presentation") {
            "### Presentation Mode\nOptimized for slides with large text and fade transitions."
        } else if current_line.contains("mode: editor") {
            "### Editor Mode\nDisplays a code editor window with syntax highlighting and line numbers."
        } else if current_line.contains("mode: terminal") {
            "### Terminal Mode\nSimulates a command-line interface session."
        } else {
            return Ok(None);
        };

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content.to_string(),
            }),
            range: None,
        }))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: DashMap::new(),
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
}

