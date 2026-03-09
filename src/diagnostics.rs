use tower_lsp::lsp_types::*;
use tree_sitter::Tree;

pub fn collect_diagnostics(_tree: &Tree, _source: &str) -> Vec<Diagnostic> {
    Vec::new()
}
