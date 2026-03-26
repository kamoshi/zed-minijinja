# zed-minijinja

Jinja2 template support for the [Zed](https://zed.dev) editor.

- Syntax highlighting for `.jinja`, `.jinja2`, and `.j2` files
- Autocompletion, hover docs, and diagnostics via [jinja-lsp](https://github.com/uros-5/jinja-lsp)

## Configuration

The language server looks for templates and backend source files based on paths you configure in Zed's LSP settings:

```json
{
  "lsp": {
    "jinja-lsp": {
      "settings": {
        "templates": "./templates",
        "backend": ["./src"],
        "lang": "rust"
      }
    }
  }
}
```

Supported `lang` values: `rust`, `python`.
