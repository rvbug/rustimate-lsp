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


    if line.ends_with("{") {
        return vec![];
    }


    if line.starts_with("transition:") || line.starts_with("transition: ") {
        return vec![
            value("fade"),
        ];
    }

    if line.starts_with("mode:") || line.starts_with("mode: ") {
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

    if line == "scene" || line == "scene " {
        return vec![
            CompletionItem {
                label: "\"scene name\" {".into(),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                insert_text: Some("\"${1:name}\" {\n\t$0\n}".into()),
                detail: Some("Create a new scene".into()),
                ..Default::default()
            }
        ];
    }

    match context {
        BlockContext::Top => vec![
            keyword("scene"),
            keyword("config"),
            scene_snippet(),
        ],

BlockContext::Scene => {

    // nothing typed → don't suggest anything
    if line.is_empty() {
        return vec![];
    }

    let mut items = Vec::new();

    if line.starts_with("m") {
        items.push(property("mode"));
    }

    if line.starts_with("a") {
        items.push(property("animation"));
    }

    if line.starts_with("e") {
        items.push(property("editor"));
    }

    if line.starts_with("t") {
        items.push(property("theme"));
    }

    if line.starts_with("c") {
        items.push(keyword("code"));
    }

    if line.starts_with("tr") {
        items.push(keyword("transition"));
    }
         

            
    // mode-dependent suggestions
    if let Some(mode) = mode {

        match mode.as_str() {
            "presentation" => {
                if line.starts_with("te") {
                   items.push(text_snippet());
                }

                if line.starts_with("tr") {
                    items.push(property("transition"));
                }
            }

            "editor" => {
                if line.starts_with("th") {
                    items.push(property("theme"));
                }

                if line.starts_with("ed") {
                    items.push(property("editor"));
                }

                if line.starts_with("co") {
                    items.push(keyword("code"));
                }
            }

            "terminal" => {
                if line.starts_with("ter") {
                    items.push(keyword("terminal"));
                }
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
            // keyword("scene"),
            // keyword("config"),
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

fn text_snippet() -> CompletionItem {
    CompletionItem {
        label: "text".into(),
        kind: Some(CompletionItemKind::SNIPPET),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        insert_text: Some("text \"${1:Hello world}\"".into()),
        detail: Some("Display text in presentation mode".into()),
        ..Default::default()
    }
}
