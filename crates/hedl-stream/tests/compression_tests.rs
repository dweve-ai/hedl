// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for compression support in hedl-stream.

// Test data - a minimal valid HEDL document
#[allow(dead_code)]
const HEDL_DOC: &str = r"%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
  | carol, Carol Williams, carol@example.com
";

#[cfg(feature = "compression")]
#[test]
fn test_streaming_parser_with_gzip_compressed_data() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use hedl_stream::{NodeEvent, StreamingParser};
    use std::io::{Cursor, Write};

    // Compress the HEDL document
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(HEDL_DOC.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();

    // Parse it through compression reader
    let cursor = Cursor::new(compressed);
    let comp_reader = hedl_stream::CompressionReader::new(cursor).unwrap();

    assert!(comp_reader.format().is_compressed());
    assert_eq!(comp_reader.format(), hedl_stream::CompressionFormat::Gzip);

    // Create parser
    let parser = StreamingParser::new(comp_reader).unwrap();

    // Collect all nodes
    let nodes: Vec<_> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| match e {
            NodeEvent::Node(n) => Some(n),
            _ => None,
        })
        .collect();

    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].id, "alice");
    assert_eq!(nodes[1].id, "bob");
    assert_eq!(nodes[2].id, "carol");
}

#[cfg(feature = "compression")]
#[test]
fn test_streaming_parser_with_uncompressed_data() {
    use hedl_stream::{NodeEvent, StreamingParser};
    use std::io::Cursor;

    // Parse uncompressed through compression reader (should auto-detect)
    let cursor = Cursor::new(HEDL_DOC);
    let comp_reader = hedl_stream::CompressionReader::new(cursor).unwrap();

    assert!(!comp_reader.format().is_compressed());
    assert_eq!(comp_reader.format(), hedl_stream::CompressionFormat::None);

    let parser = StreamingParser::new(comp_reader).unwrap();

    let nodes: Vec<_> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| match e {
            NodeEvent::Node(n) => Some(n),
            _ => None,
        })
        .collect();

    assert_eq!(nodes.len(), 3);
}

#[cfg(feature = "compression-zstd")]
#[test]
fn test_streaming_parser_with_zstd_compressed_data() {
    use hedl_stream::{NodeEvent, StreamingParser};
    use std::io::Cursor;

    // Compress the HEDL document with ZSTD
    let compressed = zstd::encode_all(HEDL_DOC.as_bytes(), 3).unwrap();

    // Parse it through compression reader
    let cursor = Cursor::new(compressed);
    let comp_reader = hedl_stream::CompressionReader::new(cursor).unwrap();

    assert!(comp_reader.format().is_compressed());
    assert_eq!(comp_reader.format(), hedl_stream::CompressionFormat::Zstd);

    let parser = StreamingParser::new(comp_reader).unwrap();

    let nodes: Vec<_> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| match e {
            NodeEvent::Node(n) => Some(n),
            _ => None,
        })
        .collect();

    assert_eq!(nodes.len(), 3);
}

#[cfg(feature = "compression-lz4")]
#[test]
fn test_streaming_parser_with_lz4_compressed_data() {
    use hedl_stream::{NodeEvent, StreamingParser};
    use lz4_flex::frame::FrameEncoder;
    use std::io::{Cursor, Write};

    // Compress the HEDL document with LZ4
    let mut encoder = FrameEncoder::new(Vec::new());
    encoder.write_all(HEDL_DOC.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();

    // Parse it through compression reader
    let cursor = Cursor::new(compressed);
    let comp_reader = hedl_stream::CompressionReader::new(cursor).unwrap();

    assert!(comp_reader.format().is_compressed());
    assert_eq!(comp_reader.format(), hedl_stream::CompressionFormat::Lz4);

    let parser = StreamingParser::new(comp_reader).unwrap();

    let nodes: Vec<_> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| match e {
            NodeEvent::Node(n) => Some(n),
            _ => None,
        })
        .collect();

    assert_eq!(nodes.len(), 3);
}

#[cfg(feature = "compression")]
#[test]
fn test_compression_format_detection_from_path() {
    use hedl_stream::CompressionFormat;

    // GZIP extensions
    assert_eq!(
        CompressionFormat::from_path("data.hedl.gz"),
        CompressionFormat::Gzip
    );
    assert_eq!(
        CompressionFormat::from_path("data.gzip"),
        CompressionFormat::Gzip
    );

    // No compression
    assert_eq!(
        CompressionFormat::from_path("data.hedl"),
        CompressionFormat::None
    );
    assert_eq!(
        CompressionFormat::from_path("data.txt"),
        CompressionFormat::None
    );
}

#[cfg(feature = "compression-zstd")]
#[test]
fn test_zstd_format_detection_from_path() {
    use hedl_stream::CompressionFormat;

    assert_eq!(
        CompressionFormat::from_path("data.hedl.zst"),
        CompressionFormat::Zstd
    );
    assert_eq!(
        CompressionFormat::from_path("data.zstd"),
        CompressionFormat::Zstd
    );
}

#[cfg(feature = "compression-lz4")]
#[test]
fn test_lz4_format_detection_from_path() {
    use hedl_stream::CompressionFormat;

    assert_eq!(
        CompressionFormat::from_path("data.lz4"),
        CompressionFormat::Lz4
    );
}

#[cfg(feature = "compression")]
#[test]
fn test_compression_writer_roundtrip() {
    use hedl_stream::{
        CompressionFormat, CompressionReader, CompressionWriter, NodeEvent, StreamingParser,
    };
    use std::io::Cursor;

    use std::io::Write;

    // Write compressed
    let mut writer = CompressionWriter::new(Vec::new(), CompressionFormat::Gzip).unwrap();
    write!(writer, "{HEDL_DOC}").unwrap();
    let compressed = writer.finish().unwrap();

    // Verify it's actually compressed (should be smaller for this text)
    assert!(compressed.len() < HEDL_DOC.len());

    // Read back
    let reader = CompressionReader::new(Cursor::new(compressed)).unwrap();
    let parser = StreamingParser::new(reader).unwrap();

    let nodes: Vec<_> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| match e {
            NodeEvent::Node(n) => Some(n),
            _ => None,
        })
        .collect();

    assert_eq!(nodes.len(), 3);
}

#[cfg(feature = "compression")]
#[test]
fn test_explicit_format_specification() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use hedl_stream::{CompressionFormat, CompressionReader, NodeEvent, StreamingParser};
    use std::io::{Cursor, Write};

    // Create gzip data but don't use magic bytes detection
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(HEDL_DOC.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();

    // Use explicit format specification
    let cursor = Cursor::new(compressed);
    let reader = CompressionReader::with_format(cursor, CompressionFormat::Gzip).unwrap();

    let parser = StreamingParser::new(reader).unwrap();
    let nodes: Vec<_> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| match e {
            NodeEvent::Node(n) => Some(n),
            _ => None,
        })
        .collect();

    assert_eq!(nodes.len(), 3);
}

#[cfg(feature = "compression")]
#[test]
fn test_compression_with_empty_document() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use hedl_stream::{CompressionReader, StreamingParser};
    use std::io::{Cursor, Write};

    // Empty HEDL document
    let empty_doc = "%VERSION: 1.0\n---\n";

    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(empty_doc.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();

    let reader = CompressionReader::new(Cursor::new(compressed)).unwrap();
    let parser = StreamingParser::new(reader).unwrap();

    // Should have no nodes
    let nodes: Vec<_> = parser.filter_map(std::result::Result::ok).collect();
    assert!(
        nodes.is_empty()
            || nodes
                .iter()
                .all(|e| !matches!(e, hedl_stream::NodeEvent::Node(_)))
    );
}

#[cfg(feature = "compression")]
#[test]
fn test_compression_writer_levels() {
    use hedl_stream::{CompressionFormat, CompressionWriter};
    use std::io::Write;

    // Test different compression levels
    for level in [1, 6, 9] {
        let mut writer =
            CompressionWriter::with_level(Vec::new(), CompressionFormat::Gzip, Some(level))
                .unwrap();
        write!(writer, "{HEDL_DOC}").unwrap();
        let compressed = writer.finish().unwrap();

        // All levels should produce valid GZIP
        assert!(!compressed.is_empty());
        // Verify magic bytes
        assert_eq!(compressed[0], 0x1f);
        assert_eq!(compressed[1], 0x8b);
    }
}
