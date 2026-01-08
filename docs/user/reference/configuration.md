# Configuration Reference

Complete reference for HEDL configuration options and environment variables.

## Environment Variables

### Resource Limits

#### `HEDL_MAX_FILE_SIZE`

Maximum file size to process.

**Default:** `1073741824` (1GB)

**Valid Values:** Positive integer (bytes)

**Example:**
```bash
# Allow 5GB files
export HEDL_MAX_FILE_SIZE=5368709120
hedl validate large.hedl
```

**Purpose:** Prevent out-of-memory errors from extremely large files.

---


### Performance Tuning

#### `RAYON_NUM_THREADS`

Control thread count for parallel batch operations (uses Rayon library).

**Default:** CPU core count

**Valid Values:** Positive integer

**Example:**
```bash
# Control parallelism
export RAYON_NUM_THREADS=4
hedl batch-validate *.hedl --parallel
```

**Note:** This is a Rayon library environment variable, not HEDL-specific.

---


## Configuration Files

### `.hedlrc` (Planned Feature)

Configuration file support is planned for a future release. Currently, all configuration
must be done via environment variables and command-line options.

**Planned features:**
- YAML or TOML format support
- Per-project configuration (`./.hedlrc`)
- User-wide configuration (`~/.hedlrc`)
- Configuration precedence hierarchy

---

## Security Settings

### DOS Protection

HEDL includes built-in protection against denial-of-service attacks:

**Resource limits:**
- File size limit
- Nesting depth limit
- String length limit
- Total key count limit

### Safe Defaults

Default configuration is designed for:
- **Reasonable limits** - Handle common files without issues
- **Security** - Prevent resource exhaustion
- **Flexibility** - Override for special cases

---

## Performance Tuning Guide

### Small Files (<10MB)

**Recommended settings:**
```bash
# Use defaults
hedl validate small.hedl
```

### Medium Files (10-100MB)

**Recommended settings:**
```bash
# Default
hedl validate medium.hedl
```

### Many Files

**Recommended settings:**
```bash
export RAYON_NUM_THREADS=8  # Adjust for your CPU
hedl batch-validate **/*.hedl --parallel
```

---

## Monitoring and Debugging

### Enable Verbose Output

```bash
hedl validate data.hedl --verbose
```

### Log to File

```bash
hedl validate data.hedl --verbose 2> debug.log
```

### Measure Performance

```bash
# Time execution
time hedl validate data.hedl

# Memory usage (Linux)
/usr/bin/time -v hedl validate data.hedl
```

---

## Quick Reference

### Common Configurations

**Development (permissive):**
```bash
export HEDL_MAX_FILE_SIZE=10737418240  # 10GB
```

**Production (secure):**
```bash
export HEDL_MAX_FILE_SIZE=1073741824  # 1GB (default)
```

**Performance (fast):**
```bash
export RAYON_NUM_THREADS=16
hedl batch-validate **/*.hedl --parallel
```

---

**Related:**
- [CLI Commands](cli-commands.md) - Command options
- [How-To: Optimize Performance](../how-to/optimize-performance.md) - Performance tuning
