# hedl-cli

Command-line interface for working with HEDL files.

## Installation

```bash
cargo install hedl-cli
```

## Commands

### Validation

```bash
hedl validate document.hedl
hedl validate --strict document.hedl
```

### Formatting

```bash
hedl format document.hedl
hedl format --in-place document.hedl
hedl format --ditto document.hedl  # Enable ditto optimization
```

### Linting

```bash
hedl lint document.hedl
hedl lint --format json document.hedl
```

### Conversion

```bash
# To other formats
hedl to-json document.hedl -o output.json
hedl to-yaml document.hedl -o output.yaml
hedl to-xml document.hedl -o output.xml
hedl to-csv document.hedl -o output.csv
hedl to-parquet document.hedl -o output.parquet
hedl to-toon document.hedl -o output.toon

# From other formats
hedl from-json input.json -o output.hedl
hedl from-yaml input.yaml -o output.hedl
hedl from-xml input.xml -o output.hedl
hedl from-csv input.csv -o output.hedl
```

### Inspection

```bash
hedl inspect document.hedl
hedl inspect --verbose document.hedl
```

### Statistics

```bash
hedl stats document.hedl
# Shows size, token count, compression ratio
```

## License

Apache-2.0
