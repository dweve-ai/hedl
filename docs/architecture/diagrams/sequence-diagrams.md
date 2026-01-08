# Sequence Diagrams

> Interaction sequences and message flows

## Parsing Sequence

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant parse_with_limits
    participant preprocess
    participant parse_header
    participant parse_body
    participant resolve_references

    User->>CLI: Parse file.hedl
    CLI->>parse_with_limits: parse_with_limits(input, options)
    parse_with_limits->>preprocess: preprocess(input, limits)
    preprocess-->>parse_with_limits: lines
    parse_with_limits->>parse_header: parse_header(lines, limits)
    parse_header-->>parse_with_limits: header
    parse_with_limits->>parse_body: parse_body(lines, header, limits)
    parse_body-->>parse_with_limits: root
    parse_with_limits->>parse_with_limits: build Document
    parse_with_limits->>resolve_references: resolve_references(doc, strict)
    resolve_references-->>parse_with_limits: validation result
    parse_with_limits-->>CLI: Document
    CLI-->>User: success / errors
```

## Format Conversion Sequence

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant parse
    participant to_json

    User->>CLI: convert file.hedl to JSON
    CLI->>parse: parse(hedl_input)
    parse-->>CLI: Document
    CLI->>to_json: to_json(document, config)
    to_json->>to_json: convert Items to JSON
    to_json-->>CLI: json_string
    CLI->>CLI: write_file(json_string)
    CLI-->>User: conversion complete
```

## LSP Interaction Sequence

```mermaid
sequenceDiagram
    participant Editor
    participant LSP
    participant Parser
    participant Cache

    Editor->>LSP: textDocument/didOpen
    LSP->>Parser: parse(content)
    Parser-->>LSP: document
    LSP->>Cache: store(uri, document)
    LSP->>LSP: build_index(document)
    LSP->>Editor: publishDiagnostics

    Editor->>LSP: textDocument/completion
    LSP->>Cache: get(uri)
    Cache-->>LSP: document
    LSP->>LSP: get_completions(position)
    LSP-->>Editor: completionItems[]
```

## Streaming Parse Sequence

```mermaid
sequenceDiagram
    participant File
    participant StreamParser
    participant Buffer
    participant Consumer

    File->>StreamParser: open(large_file)
    loop For each chunk
        File->>StreamParser: read_chunk()
        StreamParser->>Buffer: append(chunk)
        Buffer->>StreamParser: parse_available()
        StreamParser->>StreamParser: extract_nodes()
        StreamParser->>Consumer: yield node
        Consumer->>Consumer: process(node)
    end
    File-->>StreamParser: EOF
    StreamParser-->>Consumer: complete
```

---

*Last updated: 2026-01-06*
