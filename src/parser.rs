use tree_sitter::{Parser, Tree, Node, Point};
// use tree_sitter_rustimate::LANGUAGE;
use tree_sitter_rustimate::language;
#[derive(Debug, PartialEq)]
pub enum BlockContext {
    Top,
    Scene,
    Code,
    Config,
    Unknown,
}

pub struct RustimateParser {
    parser: Parser,
}

impl RustimateParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();

        parser
            .set_language(&language())
            .expect("Error loading Rustimate grammar");

        Self { parser }
    }

    pub fn parse(&mut self, text: &str) -> Option<Tree> {
        self.parser.parse(text, None)
    }
}

pub fn node_at_position(tree: &Tree, line: usize, column: usize) -> Option<Node<'_>> {
    let root = tree.root_node();

    root.descendant_for_point_range(
        Point { row: line, column },
        Point { row: line, column },
    )
}




pub fn find_block_context(node: Node) -> BlockContext {
    let mut current = node;

    loop {
        let kind = current.kind();

        match kind {
            "scene_block" => return BlockContext::Scene,
            "code_block" => return BlockContext::Code,
            "config_block" => return BlockContext::Config,
            "source_file" => return BlockContext::Top,
            _ => {}
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    BlockContext::Unknown
}


pub fn detect_scene_mode(node: Node, source: &str) -> Option<String> {
    let mut current = node;

    loop {
        if current.kind() == "scene_block" {

            let mut cursor = current.walk();

            for child in current.children(&mut cursor) {
                if child.kind() == "mode_property" {

                    let text = child.utf8_text(source.as_bytes()).ok()?;

                    if text.contains("presentation") {
                        return Some("presentation".into());
                    }

                    if text.contains("editor") {
                        return Some("editor".into());
                    }

                    if text.contains("terminal") {
                        return Some("terminal".into());
                    }
                }
            }
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }

    None
}
