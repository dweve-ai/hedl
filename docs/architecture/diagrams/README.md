# Architecture Diagrams

> Visual representations of HEDL system architecture

## Overview

This section provides comprehensive architectural diagrams using Mermaid for visualizing system structure, data flow, component relationships, and interaction sequences.

## Diagram Catalog

- [System Overview](system-overview.md) - High-level system architecture
- [Data Flow](data-flow.md) - Data transformation pipelines
- [Component Relationships](component-relationships.md) - Component dependencies
- [Sequence Diagrams](sequence-diagrams.md) - Interaction sequences

## Quick Reference Diagrams

### High-Level Architecture

```mermaid
graph TB
    subgraph "User Interface"
        CLI[Command Line]
        IDE[IDE Integration]
        API[API Clients]
    end

    subgraph "Application Layer"
        HCLI[hedl-cli]
        HLSP[hedl-lsp]
        HMCP[hedl-mcp]
    end

    subgraph "Bindings Layer"
        FFI[hedl-ffi<br/>C Bindings]
        WASM[hedl-wasm<br/>WebAssembly]
    end

    subgraph "Format Layer"
        JSON[JSON Adapter]
        YAML[YAML Adapter]
        XML[XML Adapter]
        CSV[CSV Adapter]
    end

    subgraph "Core Layer"
        CORE[hedl-core<br/>Parser Engine]
    end

    CLI --> HCLI
    IDE --> HLSP
    API --> HMCP

    HCLI --> CORE
    HLSP --> CORE
    HMCP --> CORE

    FFI --> CORE
    WASM --> CORE

    JSON --> CORE
    YAML --> CORE
    XML --> CORE
    CSV --> CORE

    style CORE fill:#e1f5ff
```

### Parsing Pipeline

```mermaid
flowchart LR
    INPUT[Raw HEDL Bytes] --> PREPROC[Preprocess]
    PREPROC --> LINES[Line Slices]
    LINES --> HEADER[Parse Header]
    LINES --> BODY[Parse Body]
    HEADER --> DOC[Document]
    BODY --> DOC
    DOC --> REFS[Resolve References]
    REFS --> VALID[Valid Document]

    style PREPROC fill:#e1f5ff
    style HEADER fill:#e1f5ff
    style BODY fill:#e1f5ff
    style REFS fill:#fff4e6
```

## Diagram Conventions

### Colors
- **Blue (#e1f5ff)**: Core components
- **Orange (#fff4e6)**: Tool/utility components
- **Green**: Data structures
- **Gray**: External systems

### Arrow Types
- **Solid line**: Direct dependency
- **Dashed line**: Optional dependency
- **Dotted line**: Data flow

## Related Documentation

- [System Overview Diagrams](system-overview.md)
- [Data Flow Diagrams](data-flow.md)
- [Component Diagrams](component-relationships.md)
- [Sequence Diagrams](sequence-diagrams.md)

---

