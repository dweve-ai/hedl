// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Transparent compression support for streaming HEDL parsing.
//!
//! This module provides automatic compression format detection and transparent
//! decompression for HEDL files. Supported formats:
//!
//! - **GZIP** (`.gz`, `.gzip`) - Wide compatibility, HTTP standard
//! - **ZSTD** (`.zst`, `.zstd`) - Best compression ratio/speed balance (optional)
//! - **LZ4** (`.lz4`) - Fastest decompression (optional)
//!
//! # Examples
//!
//! ```rust,no_run
//! use hedl_stream::compression::{CompressionFormat, CompressionReader};
//! use std::fs::File;
//!
//! // Auto-detect from file extension
//! let format = CompressionFormat::from_path("data.hedl.gz");
//! assert!(matches!(format, CompressionFormat::Gzip));
//! ```

use std::io::{self, Read};
use std::path::Path;

/// Compression format for HEDL files.
///
/// Detected automatically from file extension or magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionFormat {
    /// No compression (plain HEDL text).
    #[default]
    None,

    /// GZIP compression (RFC 1952).
    #[cfg(feature = "compression")]
    Gzip,

    /// Zstandard compression (RFC 8878).
    #[cfg(feature = "compression-zstd")]
    Zstd,

    /// LZ4 frame compression.
    #[cfg(feature = "compression-lz4")]
    Lz4,
}

impl CompressionFormat {
    /// Detect compression format from file path extension.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        match path.as_ref().extension().and_then(|s| s.to_str()) {
            #[cfg(feature = "compression")]
            Some("gz" | "gzip") => CompressionFormat::Gzip,

            #[cfg(feature = "compression-zstd")]
            Some("zst" | "zstd") => CompressionFormat::Zstd,

            #[cfg(feature = "compression-lz4")]
            Some("lz4") => CompressionFormat::Lz4,

            _ => CompressionFormat::None,
        }
    }

    /// Detect compression format from magic bytes.
    #[must_use]
    pub fn from_magic_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 2 {
            return CompressionFormat::None;
        }

        #[cfg(feature = "compression")]
        if bytes[0] == 0x1f && bytes[1] == 0x8b {
            return CompressionFormat::Gzip;
        }

        if bytes.len() >= 4 {
            #[cfg(feature = "compression-zstd")]
            if bytes[0] == 0x28 && bytes[1] == 0xb5 && bytes[2] == 0x2f && bytes[3] == 0xfd {
                return CompressionFormat::Zstd;
            }

            #[cfg(feature = "compression-lz4")]
            if bytes[0] == 0x04 && bytes[1] == 0x22 && bytes[2] == 0x4d && bytes[3] == 0x18 {
                return CompressionFormat::Lz4;
            }
        }

        CompressionFormat::None
    }

    /// Returns whether compression is enabled for this format.
    #[must_use]
    pub fn is_compressed(&self) -> bool {
        !matches!(self, CompressionFormat::None)
    }

    /// Returns the file extension typically used for this format.
    #[must_use]
    pub fn extension(&self) -> Option<&'static str> {
        match self {
            CompressionFormat::None => None,
            #[cfg(feature = "compression")]
            CompressionFormat::Gzip => Some("gz"),
            #[cfg(feature = "compression-zstd")]
            CompressionFormat::Zstd => Some("zst"),
            #[cfg(feature = "compression-lz4")]
            CompressionFormat::Lz4 => Some("lz4"),
        }
    }
}

/// A reader that transparently decompresses data based on format.
///
/// Uses boxed trait objects for simplicity and type erasure.
pub struct CompressionReader<R: Read> {
    inner: Box<dyn Read>,
    format: CompressionFormat,
    // Keep the phantom to maintain the type parameter in the signature
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Read + 'static> CompressionReader<R> {
    /// Create a compression reader with automatic format detection.
    ///
    /// Reads the first 4 bytes to detect the compression format.
    pub fn new(mut reader: R) -> io::Result<Self> {
        // Read magic bytes for format detection
        let mut magic = [0u8; 4];
        let bytes_read = Self::read_partial(&mut reader, &mut magic)?;

        // Detect format from magic bytes
        let format = CompressionFormat::from_magic_bytes(&magic[..bytes_read]);

        // Create the appropriate decoder
        Self::create_decoder(reader, format, Some(magic))
    }

    /// Create a compression reader with explicit format specification.
    pub fn with_format(reader: R, format: CompressionFormat) -> io::Result<Self> {
        Self::create_decoder(reader, format, None)
    }

    /// Get the detected or specified compression format.
    #[must_use]
    pub fn format(&self) -> CompressionFormat {
        self.format
    }

    /// Read up to `buf.len()` bytes, returning actual bytes read.
    fn read_partial(reader: &mut R, buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;
        while total < buf.len() {
            match reader.read(&mut buf[total..]) {
                Ok(0) => break,
                Ok(n) => total += n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(total)
    }

    /// Create the appropriate decoder based on format.
    fn create_decoder(
        reader: R,
        format: CompressionFormat,
        magic_prefix: Option<[u8; 4]>,
    ) -> io::Result<Self> {
        let inner: Box<dyn Read> = match (format, magic_prefix) {
            // Uncompressed - chain magic bytes back if we read them
            (CompressionFormat::None, Some(magic)) => {
                let chained = std::io::Cursor::new(magic).chain(reader);
                Box::new(chained)
            }
            (CompressionFormat::None, None) => Box::new(reader),

            // GZIP
            #[cfg(feature = "compression")]
            (CompressionFormat::Gzip, Some(magic)) => {
                let chained = std::io::Cursor::new(magic).chain(reader);
                Box::new(flate2::read::GzDecoder::new(chained))
            }
            #[cfg(feature = "compression")]
            (CompressionFormat::Gzip, None) => Box::new(flate2::read::GzDecoder::new(reader)),

            // ZSTD
            #[cfg(feature = "compression-zstd")]
            (CompressionFormat::Zstd, Some(magic)) => {
                let chained = std::io::Cursor::new(magic).chain(reader);
                let decoder = zstd::Decoder::new(chained)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Box::new(decoder)
            }
            #[cfg(feature = "compression-zstd")]
            (CompressionFormat::Zstd, None) => {
                let decoder = zstd::Decoder::new(reader)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Box::new(decoder)
            }

            // LZ4
            #[cfg(feature = "compression-lz4")]
            (CompressionFormat::Lz4, Some(magic)) => {
                let chained = std::io::Cursor::new(magic).chain(reader);
                Box::new(lz4_flex::frame::FrameDecoder::new(chained))
            }
            #[cfg(feature = "compression-lz4")]
            (CompressionFormat::Lz4, None) => Box::new(lz4_flex::frame::FrameDecoder::new(reader)),
        };

        Ok(Self {
            inner,
            format,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<R: Read> Read for CompressionReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

/// A writer that compresses data as it is written.
#[cfg(feature = "compression")]
pub struct CompressionWriter<W: std::io::Write + 'static> {
    inner: CompressionWriterInner<W>,
    format: CompressionFormat,
}

#[cfg(feature = "compression")]
enum CompressionWriterInner<W: std::io::Write> {
    Plain(W),
    // Box the large encoder types to reduce enum variant size
    Gzip(Box<flate2::write::GzEncoder<W>>),
    #[cfg(feature = "compression-zstd")]
    Zstd(Box<zstd::Encoder<'static, W>>),
    #[cfg(feature = "compression-lz4")]
    Lz4(Box<lz4_flex::frame::FrameEncoder<W>>),
}

#[cfg(feature = "compression")]
impl<W: std::io::Write + 'static> CompressionWriter<W> {
    /// Create a compression writer with the specified format.
    pub fn new(writer: W, format: CompressionFormat) -> io::Result<Self> {
        Self::with_level(writer, format, None)
    }

    /// Create a compression writer with a specific compression level.
    pub fn with_level(
        writer: W,
        format: CompressionFormat,
        level: Option<u32>,
    ) -> io::Result<Self> {
        let inner = match format {
            CompressionFormat::None => CompressionWriterInner::Plain(writer),

            CompressionFormat::Gzip => {
                let level = flate2::Compression::new(level.unwrap_or(6));
                CompressionWriterInner::Gzip(Box::new(flate2::write::GzEncoder::new(writer, level)))
            }

            #[cfg(feature = "compression-zstd")]
            CompressionFormat::Zstd => {
                let level = level.unwrap_or(3) as i32;
                CompressionWriterInner::Zstd(Box::new(
                    zstd::Encoder::new(writer, level)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
                ))
            }

            #[cfg(feature = "compression-lz4")]
            CompressionFormat::Lz4 => {
                CompressionWriterInner::Lz4(Box::new(lz4_flex::frame::FrameEncoder::new(writer)))
            }
        };

        Ok(Self { inner, format })
    }

    /// Get the compression format being used.
    pub fn format(&self) -> CompressionFormat {
        self.format
    }

    /// Finish compression and return the underlying writer.
    pub fn finish(self) -> io::Result<W> {
        match self.inner {
            CompressionWriterInner::Plain(w) => Ok(w),
            CompressionWriterInner::Gzip(w) => w.finish(),

            #[cfg(feature = "compression-zstd")]
            CompressionWriterInner::Zstd(w) => w
                .finish()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),

            #[cfg(feature = "compression-lz4")]
            CompressionWriterInner::Lz4(w) => w
                .finish()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }
}

#[cfg(feature = "compression")]
impl<W: std::io::Write + 'static> std::io::Write for CompressionWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            CompressionWriterInner::Plain(w) => w.write(buf),
            CompressionWriterInner::Gzip(w) => w.write(buf),

            #[cfg(feature = "compression-zstd")]
            CompressionWriterInner::Zstd(w) => w.write(buf),

            #[cfg(feature = "compression-lz4")]
            CompressionWriterInner::Lz4(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            CompressionWriterInner::Plain(w) => w.flush(),
            CompressionWriterInner::Gzip(w) => w.flush(),

            #[cfg(feature = "compression-zstd")]
            CompressionWriterInner::Zstd(w) => w.flush(),

            #[cfg(feature = "compression-lz4")]
            CompressionWriterInner::Lz4(w) => w.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_path_uncompressed() {
        assert_eq!(
            CompressionFormat::from_path("data.hedl"),
            CompressionFormat::None
        );
        assert_eq!(
            CompressionFormat::from_path("data.txt"),
            CompressionFormat::None
        );
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_format_from_path_gzip() {
        assert_eq!(
            CompressionFormat::from_path("data.hedl.gz"),
            CompressionFormat::Gzip
        );
    }

    #[cfg(feature = "compression-zstd")]
    #[test]
    fn test_format_from_path_zstd() {
        assert_eq!(
            CompressionFormat::from_path("data.zst"),
            CompressionFormat::Zstd
        );
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_format_from_magic_gzip() {
        assert_eq!(
            CompressionFormat::from_magic_bytes(&[0x1f, 0x8b, 0x08, 0x00]),
            CompressionFormat::Gzip
        );
    }

    #[test]
    fn test_compression_reader_uncompressed() {
        let data = b"Hello, World!";
        let reader = CompressionReader::new(std::io::Cursor::new(data.to_vec())).unwrap();
        assert_eq!(reader.format(), CompressionFormat::None);

        let mut output = String::new();
        std::io::BufReader::new(reader)
            .read_to_string(&mut output)
            .unwrap();
        // Magic bytes are chained back for uncompressed
        assert!(output.starts_with("Hell"));
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_compression_reader_gzip_roundtrip() {
        use std::io::Write;

        // Create compressed data
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(b"Hello, HEDL!").unwrap();
        let compressed = encoder.finish().unwrap();

        // Read it back
        let reader = CompressionReader::new(std::io::Cursor::new(compressed)).unwrap();
        assert_eq!(reader.format(), CompressionFormat::Gzip);

        let mut output = String::new();
        std::io::BufReader::new(reader)
            .read_to_string(&mut output)
            .unwrap();
        assert_eq!(output, "Hello, HEDL!");
    }

    #[cfg(feature = "compression")]
    #[test]
    fn test_compression_writer_gzip_roundtrip() {
        use std::io::Write;

        // Write compressed data
        let mut writer = CompressionWriter::new(Vec::new(), CompressionFormat::Gzip).unwrap();
        write!(writer, "Hello, HEDL!").unwrap();
        let compressed = writer.finish().unwrap();

        // Read it back with flate2 directly
        let mut decoder = flate2::read::GzDecoder::new(std::io::Cursor::new(compressed));
        let mut output = String::new();
        decoder.read_to_string(&mut output).unwrap();
        assert_eq!(output, "Hello, HEDL!");
    }
}
