# hedl-xml

Bidirectional XML conversion for HEDL documents.

## Installation

```toml
[dependencies]
hedl-xml = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_xml::{to_xml, from_xml, ToXmlConfig};

// HEDL to XML
let doc = parse(hedl.as_bytes())?;
let xml = to_xml(&doc)?;

// XML to HEDL
let doc = from_xml(&xml)?;

// With config
let config = ToXmlConfig::builder()
    .root_element("data")
    .pretty(true)
    .build();
let xml = hedl_xml::to_xml_with_config(&doc, &config)?;
```

## Features

- **Bidirectional conversion** - HEDL to XML and XML to HEDL
- **Streaming support** - Process large XML files
- **Schema validation** - Optional XSD validation
- **Configurable output** - Control element names and formatting

## License

Apache-2.0
