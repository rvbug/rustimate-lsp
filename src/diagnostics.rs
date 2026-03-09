
use tower_lsp::lsp_types::*;
use tree_sitter::{Tree, Node};

pub fn collect_diagnostics(tree: &Tree, source: &str) -> Vec<Diagnostic> {

    let mut diagnostics = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for node in root.children(&mut cursor) {

        if node.kind() == "scene_block" {

            if !scene_has_mode(node, source) {

                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: node.start_position().row as u32,
                            character: node.start_position().column as u32,
                        },
                        end: Position {
                            line: node.end_position().row as u32,
                            character: node.end_position().column as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Scene must declare a mode".into(),
                    source: Some("rustimate".into()),
                    ..Default::default()
                });

            }
        }
    }

    diagnostics
}

fn scene_has_mode(node: Node, source: &str) -> bool {
    if let Ok(text) = node.utf8_text(source.as_bytes()) {
        return text.contains("mode:");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    use tree_sitter_rustimate::language;

    #[test]
    fn detects_missing_mode() {
        let mut parser = Parser::new();
        parser.set_language(&language()).unwrap();

        let src = r#"
scene "hello" {
}
"#;

        let tree = parser.parse(src, None).unwrap();

        let diagnostics = collect_diagnostics(&tree, src);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn valid_scene_has_no_error() {
        let mut parser = Parser::new();
        parser.set_language(&language()).unwrap();

        let src = r#"
scene "hello" {
  mode: presentation
}
"#;

        let tree = parser.parse(src, None).unwrap();

        let diagnostics = collect_diagnostics(&tree, src);

        assert_eq!(diagnostics.len(), 0);
    }
}
