# When Things Go Wrong: A Troubleshooting Journey

Something broke. You're staring at an error message that might as well be written in ancient Sumerian. The deadline is tomorrow. Your coffee is getting cold.

Take a breath. You're not alone, and you're definitely not the first person to hit this wall.

This guide exists because every error message has a story behind it. Someone else hit that same wall, figured out what the cryptic message actually meant, and left breadcrumbs for the next traveler. That's you. Let's follow those breadcrumbs together.

---

## The Installation Gauntlet

Before you can use HEDL, you have to install it. This should be simple. Sometimes it isn't.

### "cargo install failed with compilation errors"

You ran `cargo install hedl-cli` and the terminal exploded with red text. Compiler errors. Linker errors. Something about "failed to compile."

**What's actually happening:** Rust is trying to build HEDL from source, and something in your environment is missing or outdated.

**The most common culprit: outdated Rust.**

HEDL uses modern Rust features. If your Rust installation is from six months ago, it might not understand the code.

```bash
# Check your Rust version
rustc --version

# If it's older than 1.70, update it
rustup update stable

# Now try again
cargo install hedl-cli
```

**Still failing? Your cargo cache might be corrupted.**

This happens more often than you'd think, especially if a previous installation was interrupted.

```bash
# Nuclear option: clear everything and start fresh
cargo clean
rm -rf ~/.cargo/registry/cache
cargo install hedl-cli
```

**On Linux and getting linker errors?**

You're missing system libraries. The fix depends on your distribution:

```bash
# Ubuntu or Debian
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora
sudo dnf install gcc openssl-devel

# Arch
sudo pacman -S base-devel openssl
```

**On macOS and seeing cryptic errors about missing tools?**

You need the Xcode command line tools:

```bash
xcode-select --install
```

A dialog will pop up. Click "Install." Wait. Try cargo install again.

**Nothing is working and you're losing patience?**

Try a minimal installation without optional features:

```bash
cargo install hedl-cli --no-default-features
```

This skips features that might have problematic dependencies. You can always reinstall with full features later.

---

### "hedl: command not found"

You installed HEDL successfully (or so you thought). You type `hedl --version` and the shell mocks you: "command not found."

**What's actually happening:** The `hedl` binary exists, but your shell doesn't know where to find it.

Cargo installs binaries to `~/.cargo/bin/`. Your shell needs to know to look there.

**Quick test: is the binary actually there?**

```bash
ls ~/.cargo/bin/hedl
```

If you see the file, your PATH is the problem. If not, the installation failed silently.

**Fixing your PATH (the real fix):**

Add this line to your shell configuration file:

```bash
# For bash users, add to ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"

# For zsh users, add to ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"

# For fish users, add to ~/.config/fish/config.fish
set -gx PATH $HOME/.cargo/bin $PATH
```

Then reload your shell:

```bash
source ~/.bashrc  # or ~/.zshrc, depending on your shell
```

**Need it working right now without editing files?**

Just use the full path:

```bash
~/.cargo/bin/hedl --version
```

Not elegant, but it works while you sort out your PATH.

---

### "Permission denied" during installation

The installer is trying to write somewhere it doesn't have permission to write.

**The cause is almost always: you used sudo with cargo.**

```bash
# WRONG: This causes permission nightmares
sudo cargo install hedl-cli

# RIGHT: Cargo manages its own directory, no sudo needed
cargo install hedl-cli
```

If you already made this mistake, fix the permissions:

```bash
sudo chown -R $USER:$USER ~/.cargo
```

Then install again, this time without sudo.

---

## Parsing Errors: When HEDL Doesn't Understand You

You wrote a HEDL file. You're pretty sure it's correct. The parser disagrees.

### "Unexpected token" at some line

The parser hit something it didn't expect. This is HEDL's way of saying "I'm confused."

**The most common cause: indentation.**

HEDL uses exactly one space per nesting level. Not two. Not four. Not tabs. One space.

```hedl
# WRONG: Three spaces of indentation
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
   |u1,Alice
   |u2,Bob

# CORRECT: One space per level
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |u1,Alice
 |u2,Bob
```

See the difference? In the wrong version, the pipe characters are indented three spaces. In the correct version, one space.

**Another common cause: mixing up commas and spaces.**

Matrix row values are separated by commas with no spaces:

```hedl
# WRONG: Spaces after commas
 |u1, Alice, alice@example.com

# CORRECT: No spaces after commas
 |u1,Alice,alice@example.com
```

**Special characters in strings?**

If your string contains commas, colons, or other special characters, you need quotes:

```hedl
# WRONG: The comma breaks parsing
 |m1,Hello, World,greeting

# CORRECT: Quotes protect the comma
 |m1,"Hello, World",greeting
```

**Still stuck? Use inspect to see what the parser sees:**

```bash
hedl inspect problematic.hedl
```

This shows you exactly how HEDL interprets your file, often revealing where things go wrong.

---

### "Invalid UTF-8 sequence"

Your file contains characters that aren't valid UTF-8. HEDL only speaks UTF-8.

**How did this happen?**

Usually, the file was created or edited on a system using a different encoding. Windows systems often use Windows-1252. Older Unix systems might use ISO-8859-1.

**First, check what encoding you actually have:**

```bash
file -i yourfile.hedl
# Look for the "charset=" part
```

**Convert to UTF-8:**

```bash
# From ISO-8859-1 (Latin-1)
iconv -f ISO-8859-1 -t UTF-8 yourfile.hedl > fixed.hedl

# From Windows-1252
iconv -f WINDOWS-1252 -t UTF-8 yourfile.hedl > fixed.hedl
```

**Just want to strip out the problematic characters?**

```bash
iconv -c -f UTF-8 -t UTF-8 yourfile.hedl > clean.hedl
```

The `-c` flag silently discards characters that can't be converted.

---

### "Maximum nesting depth exceeded"

Your document is nested too deeply. HEDL has limits to prevent runaway parsing and potential denial-of-service attacks.

**The default limit is 100 levels deep.** If you're hitting this, your data structure might need rethinking.

**The better solution: flatten with references.**

Instead of nesting entities inside entities inside entities, use references to link them:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Entity:[id,parent]
---
entities:@Entity
 |root,~
 |child1,@root
 |grandchild1,@child1
 |greatgrandchild1,@grandchild1
```

This flat structure can represent arbitrarily deep hierarchies without actual nesting.

**If you really need deeper nesting (library users):**

```rust
use hedl::{parse_with_limits, Limits};

let limits = Limits {
    max_nest_depth: 500,  // Increased from default 100
    ..Default::default()
};

let doc = parse_with_limits(content, limits)?;
```

But seriously, consider whether your data model needs this much nesting.

---

## Conversion Troubles: When Formats Don't Play Nice

HEDL converts to and from many formats. Sometimes the conversion doesn't do what you expected.

### "I converted to JSON and back, and my structure changed"

JSON doesn't preserve HEDL's type information. A matrix list becomes a JSON array of objects. When you convert back, HEDL has to guess at the original structure.

**Solution: Use the metadata flag for round-trip conversions.**

```bash
# Preserve structure information
hedl to-json data.hedl --metadata -o data.json

# Now the round-trip works
hedl from-json data.json -o restored.hedl
```

The `--metadata` flag embeds HEDL schema information in the JSON output.

**Better solution: Keep your HEDL as the source of truth.**

Convert to JSON for APIs and external systems, but always keep the original HEDL file. When you need to modify the data, modify the HEDL, then regenerate the JSON.

---

### "CSV import gives me wrong types" or "Where did my ID column go?"

HEDL treats the first CSV column as the ID column. This is by design: every matrix row needs an identifier.

**If your CSV doesn't have an ID column, add one:**

```csv
# Before: No ID column
name,email,age
Alice,alice@example.com,30
Bob,bob@example.com,25

# After: ID column added
id,name,email,age
u1,Alice,alice@example.com,30
u2,Bob,bob@example.com,25
```

**Numbers coming in as strings?**

Check if your CSV has quotes around numeric values:

```csv
# WRONG: Quotes make these strings
"id","age","active"
"1","30","true"

# CORRECT: No quotes for numbers and booleans
id,age,active
1,30,true
```

**Specify the type name for better results:**

```bash
hedl from-csv users.csv -t User -o users.hedl
```

The `-t` flag tells HEDL what to name the schema.

---

### "Parquet conversion failed"

Parquet is strict about types. Every value in a column must have the same type.

**The problem: Mixed types in a column.**

```hedl
# WRONG: "string" and 42 have different types
%S:Record:[id,value]
---
records:@Record
 |1,string
 |2,42
```

**The fix: Make types consistent.**

Either make everything a string (quote the numbers) or ensure all values are numeric.

```hedl
# CORRECT: All strings
%S:Record:[id,value]
---
records:@Record
 |1,string
 |2,"42"
```

Parquet also works best with flat, tabular data. If you have deeply nested structures, consider flattening them before converting.

---

## Performance: When HEDL Is Slow

HEDL is designed to be fast. If it's slow, something unusual is happening.

### "Validation takes forever on large files"

For truly large files (gigabytes), validation needs time to check every reference.

**Split and parallelize:**

```bash
# Split into 100,000-line chunks
split -l 100000 large.hedl chunk_

# Validate in parallel
for chunk in chunk_*; do
  hedl validate "$chunk" &
done
wait
```

**Use batch-validate with parallelism:**

```bash
hedl batch-validate chunks/*.hedl --parallel
```

This automatically distributes work across your CPU cores.

---

### "I'm running out of memory"

Large files need memory. If you're processing files larger than your available RAM, you'll hit limits.

**Process in chunks:**

```bash
# Split into 500MB chunks
split -b 500M large.hedl chunk_

# Process each chunk
for chunk in chunk_*; do
  hedl validate "$chunk"
done
```

**Reduce parallelism to reduce memory pressure:**

```bash
export RAYON_NUM_THREADS=2
hedl batch-validate *.hedl
```

Fewer threads mean less concurrent memory usage.

---

### "Batch operations are slower than expected"

Parallel processing has overhead. For small files, sequential processing might actually be faster.

**For many small files, try without the parallel flag:**

```bash
hedl batch-validate *.hedl  # Sequential, sometimes faster for tiny files
```

**For large files, ensure parallelism is enabled:**

```bash
hedl batch-validate *.hedl --parallel
```

**For maximum control, use GNU parallel:**

```bash
ls *.hedl | parallel -j 8 'hedl validate {}'
```

This gives you precise control over concurrency.

---

## File and I/O Headaches

### "File too large"

HEDL has a default maximum file size of 1 GB. This is a safety limit.

**For larger files, split them:**

```bash
split -b 500M huge.hedl chunk_
```

**Or convert to a more compact format:**

```bash
hedl to-parquet huge.hedl -o compact.parquet
```

Parquet is columnar and compressed. A 2 GB HEDL file might become a 200 MB Parquet file.

---

### "Permission denied"

You don't have write access to the output location.

**Check permissions:**

```bash
ls -l output.hedl
ls -ld output_directory/
```

**Fix permissions or write elsewhere:**

```bash
chmod u+w output.hedl
# Or
hedl format data.hedl -o /tmp/output.hedl
```

**Avoid sudo unless absolutely necessary.** If you need to write to a system directory, that's fine, but understand the security implications.

---

### "No such file or directory"

The file doesn't exist, or you're looking in the wrong place.

**Check your current directory:**

```bash
pwd
ls *.hedl
```

**Use absolute paths when in doubt:**

```bash
hedl validate /home/yourname/project/data.hedl
```

**Find the file:**

```bash
find . -name "*.hedl"
```

---

## Validation Failures

### "Missing required field"

A matrix row doesn't have enough values.

```hedl
# WRONG: Only 2 values, but schema has 3 columns
%S:User:[id,name,email]
---
users:@User
 |u1,Alice

# CORRECT: All 3 values present
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
```

**If a field is optional, use null:**

```hedl
 |u1,Alice,~
```

The `~` represents null (no value).

---

### "Lint warnings I don't understand"

Lint warnings aren't errors. Your file is valid. The linter is suggesting improvements.

**"Deep nesting may impact readability"**

You have many levels of indentation. Consider flattening with references.

**"Unused schema defined"**

You declared a `%S:SomeType:[...]` but never used it. Either use it or remove it.

**Warnings don't prevent your file from working.** They're suggestions. Address them when it makes sense.

---

## Batch Processing Pitfalls

### "Batch operations find 0 files"

You probably quoted the glob pattern.

```bash
# WRONG: Quotes prevent shell expansion
hedl batch-validate "*.hedl"

# CORRECT: Let the shell expand the glob
hedl batch-validate *.hedl
```

The shell needs to expand `*.hedl` into a list of actual filenames. Quotes prevent this.

---

### "I accidentally overwrote files"

Always use `--output-dir` to write formatted files to a separate directory:

```bash
hedl batch-format *.hedl --output-dir formatted/
```

**Before batch operations, use version control or backups:**

```bash
git add *.hedl
git commit -m "Before batch formatting"
hedl batch-format *.hedl --output-dir formatted/
```

---

## Platform-Specific Issues

### Windows: "Unexpected character '\r'"

Your file has Windows line endings (CRLF). HEDL expects Unix line endings (LF only).

**Convert line endings:**

```powershell
# PowerShell
(Get-Content data.hedl -Raw) -replace "`r`n", "`n" | Set-Content data.hedl -NoNewline
```

**Or use dos2unix (if installed):**

```bash
dos2unix data.hedl
```

**Prevent this in Git:**

```bash
git config --global core.autocrlf input
```

This automatically converts line endings when you check out files.

---

### Windows: Path issues

Windows uses backslashes in paths. HEDL prefers forward slashes.

```powershell
# Use forward slashes
hedl validate C:/Users/yourname/data.hedl

# Or escape backslashes
hedl validate "C:\\Users\\yourname\\data.hedl"
```

---

### macOS: SSL certificate errors during installation

The Rust toolchain is having trouble verifying SSL certificates.

```bash
# Update certificates via Homebrew
brew install openssl

# Update Rust
rustup update
```

---

### Linux: "error while loading shared libraries: libssl"

You're missing the OpenSSL library.

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# Fedora
sudo dnf install openssl-devel

# Arch
sudo pacman -S openssl
```

**For a truly portable binary, build statically:**

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

This creates a binary with no external dependencies.

---

## When All Else Fails

### Enable debug logging

```bash
export RUST_LOG=debug
hedl validate data.hedl
```

This produces verbose output that might reveal what's happening internally.

### Create a minimal reproduction

The smaller the example, the easier to debug:

```bash
# Start with the smallest possible file
cat > minimal.hedl << 'EOF'
%V:2.0
%NULL:~
%QUOTE:"
---
test:value
EOF

hedl validate minimal.hedl
```

Gradually add complexity until it breaks. That's where the bug is.

### Get help

If you've tried everything and you're still stuck:

1. **Check the FAQ:** [faq.md](faq.md)
2. **Search existing issues:** https://github.com/dweve-ai/hedl/issues
3. **Open a new issue** with:
   - Your HEDL version: `hedl --version`
   - Your operating system
   - The smallest file that reproduces the problem
   - The complete error message
   - What you expected vs. what happened

We're here to help. Every bug report makes HEDL better for everyone.

---

## Quick Reference: Error Messages and Their Meanings

| Error | Likely Cause | Quick Fix |
|-------|--------------|-----------|
| "unexpected token" | Wrong indentation or missing quotes | Check for 1-space indentation, quote special chars |
| "invalid UTF-8" | Wrong file encoding | `iconv -f ISO-8859-1 -t UTF-8 file.hedl > fixed.hedl` |
| "command not found" | PATH not set | Add `~/.cargo/bin` to PATH |
| "permission denied" | Used sudo or wrong permissions | `chown -R $USER:$USER ~/.cargo` |
| "missing required field" | Not enough values in row | Add missing values or use `~` for null |
| "maximum nesting depth" | Too many nested levels | Flatten with references |
| "file too large" | Exceeds 1 GB limit | Split file or convert to Parquet |

---

You've reached the end of the troubleshooting guide. If your problem isn't here, it's either very rare or very new. Either way, open an issue. Your problem today becomes documentation for someone else tomorrow.
