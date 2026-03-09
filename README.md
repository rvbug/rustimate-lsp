# Introduction

Rustimate is a DSL based renderer for code-based storytelling, tutorials, and presentations.

Here's the Rustimate Studio Language ("RSL") block which animates and 

scene "intro" {
  mode: presentation
  animation: typewriter
  text "Hello world"
}


# Rustimate
To know more about Rustimate check out

Introduction to [Rustimate](https://qubitai.in/rustimate.html)

It has it's own [Tree-Sitter](https://github.com/rvbug/tree-sitter-rustimate/)


# Features

* Code 
* Presentation mode
* Editor mode
* Terminal simulation
* LSP support
* Tree-sitter syntax highlighting

# Installation

Install the CLI:

`cargo install rustimate`

Install the language server:

`cargo install rustimate-lsp`


# Example

```json

scene "terminal-test" {
  mode: terminal
  animation: typewriter

  terminal """
  $ echo Hello
  Hello
  """
}

```

#  Support
Currently it supports the following Editors

| Type | Support | 
| - | - |
| Neovim |  &nbsp; ✔ |
| emacs  |  &nbsp; ✔ |
| Helix  |  &nbsp; ✔ |
| VSCode |  &nbsp; ✔ |
| Zed    |  &nbsp; ✔ |
| Lapce  |  &nbsp; ✔ |
| Markdown |  &nbsp; ✔ |
| Quarto   |  &nbsp; ✔ |


# Language Server

Rustimate includes a Language Server that provides. 

* autocomplete
* snippets
* diagnostics
* hover documentation

Please note that project is still in development. 
If you have any issues, do raise a PR 


# Reference

TBD






