use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use dashmap::DashMap;

struct Backend {
    #[allow(dead_code)]
    client: Client,
    documents: DashMap<String, String>, 
}

// Helper function to find block context (scene, code, config)
fn get_current_block(text: &str, line: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line >= lines.len() { return "top".to_string(); }

    for i in (0..=line).rev() {
        let current_line = lines[i].trim();
        
        // Check for 'code' block - must have an open brace that hasn't been closed
        if current_line.contains("code") && current_line.contains("{") { return "code".to_string(); }
        // Check for 'scene' block - must have an open brace
        if current_line.contains("scene") && current_line.contains("{") { return "scene".to_string(); }
        if current_line.contains("config") && current_line.contains("{") { return "config".to_string(); }
    }
    "top".to_string()
}



// Helper to get the exact text of the current line
fn get_line_context(text: &str, line: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line >= lines.len() { return "".to_string(); }
    lines[line].trim().to_string()
}

// Helper to create CompletionItems with optional Markdown documentation
fn create_completion(label: &str, detail: &str, doc: Option<&str>) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        documentation: doc.map(|d| Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: d.to_string(),
        })),
        kind: Some(CompletionItemKind::PROPERTY),
        ..Default::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // REQUIRED: Tells Neovim to sync the buffer content so self.documents isn't empty
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![":".to_string(), "{".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.documents.insert(params.text_document.uri.to_string(), params.text_document.text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents.insert(params.text_document.uri.to_string(), change.text);
        }
    }




async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let doc_text = match self.documents.get(&uri) {
            Some(t) => t.value().clone(),
            None => return Ok(None),
        };

        let current_line = get_line_context(&doc_text, position.line as usize);
        let block_context = get_current_block(&doc_text, position.line as usize);
        
        let mut completions = Vec::new();

        // 1. If the line is currently being used to define the scene, show nothing (allow typing name)
        if current_line.starts_with("scene") && !current_line.contains("{") {
             return Ok(None);
        }

        // 2. VALUE COMPLETIONS (e.g., after 'mode:')
        if current_line.ends_with("mode:") || current_line.ends_with("mode: ") {
            completions.push(create_completion("editor", "Standard code editor", None));
            completions.push(create_completion("presentation", "Slide view", None));
            completions.push(create_completion("terminal", "Interactive terminal", None));
        } 
        // 3. CONTEXTUAL PROPERTY COMPLETIONS (Only if inside a block)
        else {
            match block_context.as_str() {
                "scene" => {
                    // Only suggest these if we aren't on the same line as the 'scene "name" {' header
                    if !current_line.contains("scene") {
                        completions.push(create_completion("mode:", "Set display mode", None));
                        completions.push(create_completion("theme:", "Set syntax theme", None));
                        completions.push(create_completion("code", "Add a code block", None));
                    }
                },
                "code" => {
                    if !current_line.contains("code") {
                        completions.push(create_completion("file:", "Source file path", None));
                        completions.push(create_completion("lines:", "Line range", None));
                    }
                },
                _ => {
                    // Top level suggestions
                    completions.push(create_completion("scene", "Start a new scene: scene \"name\" { }", None));
                    
                    // SNIPPET: The proper way to do 'scene "name" { }'
                    completions.push(CompletionItem {
                        label: "scene-template".to_string(),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("Full scene block template".to_string()),
                        insert_text: Some("scene \"${1:name}\" {\n\t$0\n}".to_string()),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    });
                }
            }
        }

        Ok(Some(CompletionResponse::Array(completions)))
    }


    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        let doc_text = match self.documents.get(&uri) {
            Some(t) => t.value().clone(),
            None => return Ok(None),
        };

        let current_line = get_line_context(&doc_text, position.line as usize);

        // Simple Hover Implementation
        if current_line.contains("mode:") {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "### mode\nDefines the visual context for the current scene.\n- **editor**: Code with line numbers.\n- **presentation**: Clean content.".to_string(),
                }),
                range: None,
            }));
        }

        Ok(None)
    }
}


// Helper to detect the active mode within a scene block
fn get_active_mode(text: &str, line: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    for i in (0..=line).rev() {
        let l = lines[i].trim();
        if l.contains("scene") && i < line - 10 { break; } // Don't look too far back
        if l.contains("mode: presentation") { return "presentation".to_string(); }
        if l.contains("mode: editor") { return "editor".to_string(); }
        if l.contains("mode: terminal") { return "terminal".to_string(); }
    }
    "none".to_string()
}




#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { 
        client, 
        documents: DashMap::new() 
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
}
