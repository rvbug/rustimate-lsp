use tower_lsp::lsp_types::*;
use crate::parser::BlockContext;


pub fn completions(context: BlockContext, mode: Option<String>) -> Vec<CompletionItem> {
    // vec![
    //     keyword("scene"),
    //     keyword("mode"),
    //     keyword("text"),
    //     scene_snippet(),
    // ]
    //
    println!("Completion triggered");
    println!("Context: {:?}", context);
    println!("Mode: {:?}", mode);

    match context {

        BlockContext::Top => vec![
            keyword("scene"),
            keyword("config"),
            scene_snippet(),
        ],

        BlockContext::Scene => {

            let mut items = vec![
                keyword("mode"),
                keyword("animation"),
            ];

            if let Some(mode) = mode {

                match mode.as_str() {

                    "presentation" => {
                        items.push(keyword("text"));
                        items.push(keyword("transition"));
                    }

                    "editor" => {
                        items.push(keyword("theme"));
                        items.push(keyword("editor"));
                        items.push(keyword("code"));
                    }

                    "terminal" => {
                        items.push(keyword("terminal"));
                    }

                    _ => {}
                }
            }

            items
        }

        BlockContext::Code => vec![
            keyword("file"),
            keyword("lines"),
            keyword("highlight"),
        ],

        _ => vec![
            keyword("scene"),
            keyword("config"),
        ]
    }



}



fn keyword(name: &str) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(CompletionItemKind::KEYWORD),
        insert_text: Some(format!("{}:", name)),
        ..Default::default()
    }
}

fn scene_snippet() -> CompletionItem {
    CompletionItem {
        label: "scene snippet".into(),
        kind: Some(CompletionItemKind::SNIPPET),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        insert_text: Some(
            r#"scene "${1:name}" {
    mode: ${2|presentation,editor,terminal|}
}"#
            .into(),
        ),
        ..Default::default()
    }
}
