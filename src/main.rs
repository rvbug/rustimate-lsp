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

// ---------------------------------------------------------
// HELPERS
// ---------------------------------------------------------

/// Identifies if we are in the top level, a scene block, or a nested code block.
fn get_current_block(text: &str, line: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line >= lines.len() { return "top".to_string(); }

    let mut brace_stack = 0;
    for i in (0..=line).rev() {
        let current_line = lines[i].trim();
        
        // Count braces to determine nesting depth
        if current_line.contains('}') { brace_stack += 1; }
        if current_line.contains('{') {
            if brace_stack > 0 {
                brace_stack -= 1;
                continue;
            }
            if current_line.contains("code {") { return "code".to_string(); }
            if current_line.contains("scene") { return "scene".to_string(); }
            if current_line.contains("config") { return "config".to_string(); }
        }
    }
    "top".to_string()
}

/// Looks specifically for the 'mode:' setting within the current scene block.
fn get_active_mode(text: &str, line: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    for i in (0..=line).rev() {
        let l = lines[i].trim();
        if l.contains("mode: presentation") { return "presentation".to_string(); }
        if l.contains("mode: editor") { return "editor".to_string(); }
        if l.contains("mode: terminal") { return "terminal".to_string(); }
        // Stop if we hit the start of the scene so we don't bleed into previous scenes
        if l.contains("scene") && l.contains('{') { break; }
    }
    "none".to_string()
}

fn get_line_context(text: &str, line: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line >= lines.len() { return "".to_string(); }
    lines[line].to_string() // Keep whitespace for "scene " check
}

fn create_completion(label: &str, detail: &str, doc: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc.to_string(),
        })),
        kind: Some(CompletionItemKind::KEYWORD),
        insert_text: Some(label.to_string()),
        ..Default::default()
    }
}

// ---------------------------------------------------------
// SERVER IMPLEMENTATION
// ---------------------------------------------------------

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

   

   // async fn completion(
   //      &self,
   //      _: CompletionParams,
   //  ) -> Result<Option<CompletionResponse>> {
   //
   //      let items = completions();
   //
   //      Ok(Some(CompletionResponse::Array(items)))
   //  }


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

