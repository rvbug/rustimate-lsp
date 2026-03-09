use tower_lsp::lsp_types::*;
use crate::parser::BlockContext;

fn value(name: &str) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(CompletionItemKind::VALUE),
        insert_text: Some(name.into()),
        ..Default::default()
    }
}


pub fn completions(context: BlockContext, mode: Option<String>, line: &str) -> Vec<CompletionItem> {

    println!("Completion triggered");
    println!("Context: {:?}", context);
    println!("Mode: {:?}", mode);


    if line.starts_with("mode:") {
        return vec![
            value("presentation"),
            value("editor"),
            value("terminal"),
        ];
    }

    if line.starts_with("animation:") {
        return vec![
            value("static"),
            value("typewriter"),
        ];
    }

    if line.starts_with("editor:") {
        return vec![
            value("neovim"),
            value("emacs"),
        ];
    }

    if line.starts_with("theme:") {
        return vec![
            value("monokai"),
            value("nord"),
            value("dracula"),
        ];
    }

    if line.starts_with("terminal") {
        return vec![];
    }

    match context {

        BlockContext::Top => vec![
            keyword("scene"),
            keyword("config"),
            scene_snippet(),
        ],

        BlockContext::Scene => {

            // let mut items = vec![
            //     keyword("mode"),
            //     keyword("animation"),
            // ];

            let mut items = vec![
                property("mode"),
                property("animation"),
                property("editor"),
                property("theme"),
                keyword("code"),
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
        insert_text: Some(name.into()),
        ..Default::default()
    }
}

fn property(name: &str) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(CompletionItemKind::PROPERTY),
        insert_text: Some(format!("{}: ", name)),
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
