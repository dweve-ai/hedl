# Troubleshooting Guide

Solutions to common issues when using HEDL.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Parsing Errors](#parsing-errors)
3. [Conversion Problems](#conversion-problems)
4. [Performance Issues](#performance-issues)
5. [File and I/O Errors](#file-and-io-errors)
6. [Validation Failures](#validation-failures)
7. [Batch Processing Issues](#batch-processing-issues)
8. [Platform-Specific Issues](#platform-specific-issues)

## Installation Issues

### Problem: `cargo install` fails with compilation errors

**Symptoms:**
```
error: failed to compile hedl-cli
```

**Solutions:**

**1. Update Rust:**
```bash
rustup update stable
rustc --version  # Should be 1.70 or later
```

**2. Clear cargo cache:**
```bash
cargo clean
rm -rf ~/.cargo/registry/cache
cargo install --path crates/hedl-cli
```

**3. Install with specific features:**
```bash
# Minimal installation
cargo install hedl-cli --no-default-features
```

**4. Check system dependencies:**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install

# Fedora
sudo dnf install gcc openssl-devel
```

---

### Problem: `hedl: command not found`

**Symptoms:**
```bash
hedl --version
# hedl: command not found
```

**Solutions:**

**1. Check if cargo bin is in PATH:**
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.bashrc
```

**2. Verify installation:**
```bash
ls ~/.cargo/bin/hedl
# If not present, reinstall
cargo install --path crates/hedl-cli --force
```

**3. Use full path temporarily:**
```bash
~/.cargo/bin/hedl --version
```

---

### Problem: Permission denied during installation

**Symptoms:**
```
error: failed to create directory: Permission denied
```

**Solutions:**

**1. Don't use sudo with cargo:**
```bash
# WRONG
sudo cargo install hedl-cli

# CORRECT
cargo install hedl-cli
```

**2. Fix cargo permissions:**
```bash
sudo chown -R $USER:$USER ~/.cargo
```

**3. Install to custom location:**
```bash
cargo install --path crates/hedl-cli --root ~/my-tools
export PATH="$HOME/my-tools/bin:$PATH"
```

## Parsing Errors

### Problem: "Unexpected token" errors

**Symptoms:**
```
Error: Parse error at line 5, column 12: unexpected token
```

**Solutions:**

**1. Check syntax carefully:**
```hedl
# WRONG - missing quotes when needed
%STRUCT: User: [id, name]
---
users: @User
  | u1, "Alice with spaces"

# CORRECT - quotes not needed for simple identifiers
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

**2. Verify indentation:**
```hedl
# WRONG - inconsistent indentation
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
   | u1, Alice    # Wrong indentation
  | u2, Bob       # Correct indentation

# CORRECT - consistent 2-space indentation
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
```

**3. Check special characters:**
```hedl
# WRONG - unescaped quotes in string values
%STRUCT: Message: [id, text]
---
messages: @Message
  | m1, "She said "hello""

# CORRECT - use simple strings or escape if needed
%STRUCT: Message: [id, text]
---
messages: @Message
  | m1, She said hello
```

**4. Use `inspect` to debug:**
```bash
hedl inspect problematic.hedl
# Shows where parsing fails
```

---

### Problem: "Invalid UTF-8" errors

**Symptoms:**
```
Error: Invalid UTF-8 sequence at byte 42
```

**Solutions:**

**1. Check file encoding:**
```bash
file -i myfile.hedl
# Should show: charset=utf-8
```

**2. Convert to UTF-8:**
```bash
# From ISO-8859-1 to UTF-8
iconv -f ISO-8859-1 -t UTF-8 myfile.hedl > utf8.hedl

# From Windows-1252 to UTF-8
iconv -f WINDOWS-1252 -t UTF-8 myfile.hedl > utf8.hedl
```

**3. Remove non-UTF-8 characters:**
```bash
# Replace invalid characters
iconv -c -f UTF-8 -t UTF-8 myfile.hedl > clean.hedl
```

---

### Problem: "Maximum nesting depth exceeded"

**Symptoms:**
```
Error: Maximum nesting depth (100) exceeded
```

**Solutions:**

**1. Flatten your structure:**
```hedl
# BETTER - use references instead of deep nesting
%STRUCT: Entity: [id, parent]
---
entities: @Entity
  | e1, ~
  | e2, @Entity:e1
  | e3, @Entity:e2
  # ... flat structure with references
```

**2. Split into multiple documents:**
```bash
# Instead of one huge nested document
# Use multiple related documents
hedl validate part1.hedl
hedl validate part2.hedl
hedl validate part3.hedl
```

**3. For library users, increase limit:**
```rust
use hedl::{parse_with_limits, Limits};

let limits = Limits {
    max_nest_depth: 2000,  // Increased
    ..Default::default()
};
```

## Conversion Problems

### Problem: JSON conversion loses HEDL type information

**Symptoms:**
```bash
hedl to-json data.hedl -o output.json
hedl from-json output.json -o restored.hedl
# restored.hedl has different structure than data.hedl
```

**Solutions:**

**1. Use `--metadata` flag:**
```bash
# Preserve HEDL structure
hedl to-json data.hedl --metadata -o output.json
hedl from-json output.json -o restored.hedl
```

**2. Keep original HEDL for reference:**
```bash
# Convert for external use, keep HEDL as source of truth
hedl to-json source.hedl -o for_api.json
# Always convert from source.hedl, not from JSON
```

---

### Problem: CSV import has wrong types

**Symptoms:**
```bash
hedl from-csv data.csv -o output.hedl
# Numbers imported as strings
```

**Example:**
```csv
id,age,active
1,30,true
```

**Output (undesired):**
```hedl
%STRUCT: Record: [id, age, active]
---
records: @Record
  | 1, 30, true  # Everything is a string!
```

**Solutions:**

**1. Specify type name for the matrix list:**
```bash
# Use --type-name to set the struct name
hedl from-csv data.csv --type-name Record -o output.hedl

# Output (correct types inferred):
# %STRUCT: Record: [id, age, active]
# ---
# records: @Record
#   | 1, 30, true
```

**2. Check CSV format:**
```csv
# WRONG - numbers in quotes
"id","age","active"
"1","30","true"

# CORRECT - no quotes for numbers/booleans
id,age,active
1,30,true
```

**3. Manually fix after import:**
```bash
hedl from-csv data.csv > temp.hedl
hedl format temp.hedl -o corrected.hedl
# Then manually edit types if needed
```

---

### Problem: XML attributes are lost or mangled

**Symptoms:**
```xml
<book id="b1" format="hardcover">
  <title>Example</title>
</book>
```

Converts to unexpected structure.

**Solutions:**

**1. Understand attribute conversion:**
XML attributes become `_attr_` prefixed fields:

```hedl
book @Book 1
  _attr_id "b1"
  _attr_format "hardcover"
  title "Example"
```

**2. Convert back preserves attributes:**
```bash
hedl from-xml data.xml -o data.hedl
hedl to-xml data.hedl -o restored.xml
# Attributes are preserved
```

**3. For clean structure, preprocess XML:**
```bash
# Use XSLT or xmlstarlet to convert attributes to elements first
xmlstarlet tr attributes-to-elements.xsl data.xml | hedl from-xml -
```

---

### Problem: Parquet conversion fails

**Symptoms:**
```
Error: Parquet conversion failed: schema inference error
```

**Solutions:**

**1. Ensure consistent types:**
```hedl
# WRONG - mixed types in same field
%STRUCT: Record: [id, value]
---
records: @Record
  | 1, string
  | 2, 42

# CORRECT - consistent types
%STRUCT: Record: [id, value]
---
records: @Record
  | 1, string
  | 2, 42_as_string
```

**2. Use simpler structures:**
Parquet works best with flat, tabular data:

```hedl
# GOOD for Parquet
%STRUCT: User: [id, name, age]
---
users: @User
  | 1, Alice, 30
  | 2, Bob, 25
  | 3, Charlie, 35
```

**3. Check error details:**
```bash
hedl to-parquet data.hedl -o output.parquet 2>&1 | tee error.log
```

## Performance Issues

### Problem: Slow validation of large files

**Symptoms:**
```bash
time hedl validate large.hedl
# real 2m30.000s (too slow!)
```

**Solutions:**

**1. Split into smaller files:**
```bash
# Split large file
split -l 100000 large.hedl chunk_

# Validate in parallel
for chunk in chunk_*; do
  hedl validate "$chunk" &
done
wait
```

**2. Use batch processing:**
```bash
# Parallel validation (shell expands glob)
hedl batch-validate chunks/*.hedl --parallel
```

**3. Check for performance issues:**
```bash
# Profile the operation (Linux)
perf record hedl validate large.hedl
perf report
```

---

### Problem: High memory usage

**Symptoms:**
```
Out of memory error or system slowdown
```

**Solutions:**

**1. Split very large files:**
```bash
# For files > 1GB, split into smaller chunks
split -b 500M large.hedl chunk_

# Process each chunk
for chunk in chunk_*; do
  hedl validate "$chunk"
done
```

**2. Reduce batch size:**
```bash
# Process fewer files at once
export RAYON_NUM_THREADS=2  # Reduce parallelism
hedl batch-validate *.hedl
```

**3. Monitor memory:**
```bash
# Linux
/usr/bin/time -v hedl validate large.hedl

# macOS
time hedl validate large.hedl
```

---

### Problem: Batch operations are slow

**Symptoms:**
```bash
hedl batch-format *.hedl --output-dir formatted/
# Takes much longer than expected
```

**Solutions:**

**1. Ensure parallelism is enabled:**
```bash
# Explicitly enable (default)
hedl batch-format *.hedl --output-dir formatted/ --parallel
```

**2. Adjust thread count:**
```bash
# Use all cores
export RAYON_NUM_THREADS=$(nproc)

# Or specific count
export RAYON_NUM_THREADS=8
```

**3. For many small files, omit parallel flag:**
```bash
# Sometimes faster for small files (default behavior for <4 files)
hedl batch-format *.hedl --output-dir formatted/
```

**4. Use GNU parallel for maximum control:**
```bash
ls *.hedl | parallel -j 8 'hedl format {} -o formatted/{}'
```

## File and I/O Errors

### Problem: "File too large" error

**Symptoms:**
```
Error: File 'large.hedl' is too large (2000000000 bytes).
Maximum allowed size is 1073741824 bytes (1024 MB).
```

**Solutions:**

**1. Split file:**
```bash
# Split into 500MB chunks
split -b 500M large.hedl chunk_

# Process chunks
for chunk in chunk_*; do
  hedl validate "$chunk"
done
```

**2. Compress data:**
```bash
# Convert to more efficient format
hedl to-parquet large.hedl -o compressed.parquet
# Parquet is typically 70-90% smaller
```

---

### Problem: "Permission denied" errors

**Symptoms:**
```
Error: Failed to write 'output.hedl': Permission denied
```

**Solutions:**

**1. Check file permissions:**
```bash
ls -l output.hedl
# Fix permissions
chmod u+w output.hedl
```

**2. Check directory permissions:**
```bash
ls -ld output_directory/
# Fix directory permissions
chmod u+w output_directory/
```

**3. Write to different location:**
```bash
hedl format data.hedl -o /tmp/output.hedl
```

**4. Use sudo only if necessary:**
```bash
# Avoid sudo with HEDL when possible
# If required, use carefully
sudo hedl format data.hedl -o /etc/config.hedl
```

---

### Problem: "No such file or directory"

**Symptoms:**
```
Error: Failed to read 'data.hedl': No such file or directory
```

**Solutions:**

**1. Check file path:**
```bash
ls -l data.hedl
# Use absolute path if needed
hedl validate /full/path/to/data.hedl
```

**2. Check current directory:**
```bash
pwd
ls *.hedl
```

**3. Use find to locate file:**
```bash
find . -name "data.hedl"
```

**4. For batch operations, let shell expand glob:**
```bash
# Shell expands glob pattern
hedl batch-validate *.hedl

# Test pattern first
ls *.hedl
```

## Validation Failures

### Problem: "Missing required field" errors

**Symptoms:**
```
Error: Validation failed: Missing required field 'name'
```

**Solutions:**

**1. Check matrix list structure:**
```hedl
# WRONG - missing field
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice  # Missing email!

# CORRECT
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
```

**2. Use `inspect` to see structure:**
```bash
hedl inspect data.hedl
```

**3. Add missing fields:**
```hedl
# Use null (~) for optional fields
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, ~
```

---

### Problem: Lint warnings you don't understand

**Symptoms:**
```
Warning (line 12): Deep nesting (level 8) may impact readability
```

**Solutions:**

**1. Understand the warning:**
- Deep nesting: Refactor to flatten structure or use references
- Unused types: Remove or reference them
- Missing count hints: Add count for better performance

**2. Fix or ignore:**
```bash
# View all warnings
hedl lint data.hedl

# Warnings don't prevent usage
hedl validate data.hedl  # May still be valid
```

**3. Refactor if needed:**
```hedl
# AFTER - flattened with references
%STRUCT: L3: [id]
%STRUCT: L2: [id, child]
%STRUCT: L1: [id, child]
---
level3_items: @L3
  | l3_1

level2_items: @L2
  | l2_1, @L3:l3_1

level1_items: @L1
  | l1_1, @L2:l2_1
```

## Batch Processing Issues

### Problem: Batch operations don't find files

**Symptoms:**
```bash
hedl batch-validate "*.hedl"
# Processed 0 files
```

**Solutions:**

**1. Let shell expand glob pattern:**
```bash
# Shell expands the glob - do NOT quote
hedl batch-validate *.hedl
```

**2. Check files exist:**
```bash
ls *.hedl
```

**3. Use absolute paths:**
```bash
hedl batch-validate /full/path/*.hedl
```

**4. Try different glob patterns:**
```bash
# Specific directory
hedl batch-validate data/*.hedl

# Multiple patterns
hedl batch-validate data/*.hedl archive/*.hedl
```

---

### Problem: Batch operations modify wrong files

**Symptoms:**
```bash
hedl batch-format *.hedl --output-dir .
# Oops, modified files I didn't want to change!
```

**Solutions:**

**1. Always use --output-dir to avoid overwriting:**
```bash
# Output to different directory
hedl batch-format *.hedl --output-dir formatted/
```

**2. Backup before batch operations:**
```bash
tar czf backup.tar.gz *.hedl
hedl batch-format *.hedl --output-dir formatted/
```

**3. Use version control:**
```bash
git add *.hedl
git commit -m "Before batch format"
hedl batch-format *.hedl --output-dir formatted/
git diff  # Review changes
```

## Platform-Specific Issues

### Windows: Line ending issues

**Symptoms:**
```
Parse error: unexpected character '\r'
```

**Solutions:**

**1. Convert line endings:**
```powershell
# PowerShell
(Get-Content data.hedl) | Set-Content -NoNewline data.hedl
```

**2. Use Git autocrlf:**
```bash
git config --global core.autocrlf input
```

**3. Use dos2unix:**
```bash
dos2unix data.hedl
```

---

### Windows: Path issues

**Symptoms:**
```
Error: Failed to read 'C:\path\data.hedl'
```

**Solutions:**

**1. Use forward slashes:**
```powershell
hedl validate C:/path/data.hedl
```

**2. Escape backslashes:**
```powershell
hedl validate "C:\\path\\data.hedl"
```

**3. Use PowerShell:**
```powershell
hedl validate $PWD\data.hedl
```

---

### macOS: SSL certificate errors (during installation)

**Symptoms:**
```
error: failed to get '...' from registry
```

**Solutions:**

**1. Update certificates:**
```bash
brew install openssl
```

**2. Update Rust:**
```bash
rustup update
```

**3. Use different registry:**
```bash
cargo install hedl-cli --registry crates-io
```

---

### Linux: Library linking errors

**Symptoms:**
```
error while loading shared libraries: libssl.so.1.1
```

**Solutions:**

**1. Install OpenSSL:**
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# Fedora
sudo dnf install openssl-devel

# Arch
sudo pacman -S openssl
```

**2. Build static binary:**
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

## Getting More Help

### Debug Mode

Enable verbose logging:

```bash
# Set log level
export RUST_LOG=debug
hedl validate data.hedl
```

### Create Minimal Reproduction

Create the smallest example that reproduces the issue:

```bash
# Minimal file
echo -e '%VERSION: 1.0\n---\nname: test' > minimal.hedl

# Test it
hedl validate minimal.hedl
```

### Report Issues

If you can't solve the problem:

1. **Check FAQ**: [faq.md](faq.md)
2. **Search Issues**: https://github.com/dweve-ai/hedl/issues
3. **Create Issue**: Include:
   - HEDL version: `hedl --version`
   - OS and version
   - Minimal reproduction case
   - Error messages (full output)
   - What you expected vs what happened

### Community Resources

- **GitHub Issues**: https://github.com/dweve-ai/hedl/issues
- **Documentation**: This guide and [README.md](README.md)
- **Examples**: [examples.md](examples.md)
- **CLI Reference**: [cli-guide.md](cli-guide.md)

---

**Still stuck?** Create a detailed issue on GitHub with:
- `hedl --version` output
- Your operating system
- Minimal example that reproduces the problem
- Full error message

We're here to help!
