# HEDL Examples and Use Cases

Practical examples demonstrating common HEDL usage patterns and real-world scenarios.

## Table of Contents

1. [Basic Examples](#basic-examples)
2. [Data Conversion](#data-conversion)
3. [AI/ML Workflows](#aiml-workflows)
4. [Data Processing Pipelines](#data-processing-pipelines)
5. [Configuration Management](#configuration-management)
6. [Graph Data](#graph-data)
7. [Advanced Patterns](#advanced-patterns)

## Basic Examples

### Example 1: Simple User List

Create a structured list of users with typed fields.

**HEDL:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email, age]
---
users: @User
  | u1, Alice Johnson, alice@example.com, 30
  | u2, Bob Smith, bob@example.com, 25
  | u3, Charlie Davis, charlie@example.com, 35
```

**Commands:**
```bash
# Validate
hedl validate users.hedl

# Convert to JSON
hedl to-json users.hedl --pretty -o users.json

# Check token efficiency
hedl stats users.hedl
```

**JSON Output:**
```json
{
  "users": [
    {
      "id": "u1",
      "name": "Alice Johnson",
      "email": "alice@example.com",
      "age": 30
    },
    {
      "id": "u2",
      "name": "Bob Smith",
      "email": "bob@example.com",
      "age": 25
    },
    {
      "id": "u3",
      "name": "Charlie Davis",
      "email": "charlie@example.com",
      "age": 35
    }
  ]
}
```

### Example 2: Nested Structures

Represent hierarchical data with nested entities.

**HEDL:**
```hedl
%VERSION: 1.0
%STRUCT: Department: [id, name, headcount]
---
company:
  name: Acme Corp
  founded: 2010
  departments: @Department
    | d1, Engineering, 50
    | d2, Sales, 30
    | d3, Marketing, 20
  headquarters:
    street: 123 Main St
    city: San Francisco
    state: CA
    zip: 94105
```

**Commands:**
```bash
# Format nicely
hedl format company.hedl -o company_formatted.hedl

# Convert to YAML
hedl to-yaml company.hedl -o company.yaml
```

### Example 3: Using the Ditto Operator

Efficiently repeat values using the `^` operator.

**HEDL:**
```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, priority, assignee]
---
tasks: @Task
  | t1, pending, high, Alice
  | t2, ^, ^, Bob
  | t3, ^, medium, Alice
  | t4, complete, low, ^
  | t5, pending, high, Charlie
```

**Equivalent (expanded):**
```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, priority, assignee]
---
tasks: @Task
  | t1, pending, high, Alice
  | t2, pending, high, Bob
  | t3, pending, medium, Alice
  | t4, complete, low, Alice
  | t5, pending, high, Charlie
```

**Benefit:** 25% fewer tokens than fully expanded form.

### Example 4: Entity References

Link entities using references (`@Type:id`).

**HEDL:**
```hedl
%VERSION: 1.0
%STRUCT: Author: [id, name]
%STRUCT: Book: [id, title, author, year]
---
authors: @Author
  | a1, Margaret Atwood
  | a2, Neil Gaiman

books: @Book
  | b1, The Handmaid's Tale, @Author:a1, 1985
  | b2, American Gods, @Author:a2, 2001
  | b3, Good Omens, @Author:a2, 1990
```

**Use Case:** Maintain referential integrity and avoid data duplication.

## Data Conversion

### Example 5: CSV Import with Auto-Schema

Import CSV data with automatic type inference.

**Input (customers.csv):**
```csv
id,name,email,age,active,balance
1,Alice,alice@example.com,30,true,1250.50
2,Bob,bob@example.com,25,false,0.00
3,Charlie,charlie@example.com,35,true,3500.75
```

**Commands:**
```bash
# Import with auto-schema inference
hedl from-csv customers.csv --type-name Customer -o customers.hedl

# Validate the result
hedl validate customers.hedl

# Export to Parquet for analytics
hedl to-parquet customers.hedl -o customers.parquet
```

**Output (customers.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: Customer: [id, name, email, age, active, balance]
---
customers: @Customer
  | 1, Alice, alice@example.com, 30, true, 1250.5
  | 2, Bob, bob@example.com, 25, false, 0.0
  | 3, Charlie, charlie@example.com, 35, true, 3500.75
```

### Example 6: JSON API Response Processing

Process JSON from an API and convert to token-efficient HEDL.

**Input (api_response.json):**
```json
{
  "status": "success",
  "data": {
    "users": [
      {"id": 1, "username": "alice", "role": "admin"},
      {"id": 2, "username": "bob", "role": "user"},
      {"id": 3, "username": "charlie", "role": "user"}
    ],
    "total": 3
  }
}
```

**Commands:**
```bash
# Fetch and convert
curl -s https://api.example.com/users > api_response.json
hedl from-json api_response.json -o users.hedl

# Check token savings
hedl stats users.hedl
```

**Output (users.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, username, role]
---
status: success
data:
  users: @User
    | 1, alice, admin
    | 2, bob, user
    | 3, charlie, user
  total: 3
```

**Savings:** Typically 40-60% fewer tokens than JSON.

### Example 7: Multi-Format Conversion Pipeline

Convert data through multiple formats for different use cases.

**Workflow:**
```bash
#!/bin/bash
# multi_convert.sh - Convert data to all supported formats

INPUT="data.hedl"
BASE="${INPUT%.hedl}"

# Validate source
echo "Validating source..."
hedl validate "$INPUT" || exit 1

# Convert to all formats
echo "Converting to JSON..."
hedl to-json "$INPUT" --pretty -o "${BASE}.json"

echo "Converting to YAML..."
hedl to-yaml "$INPUT" -o "${BASE}.yaml"

echo "Converting to XML..."
hedl to-xml "$INPUT" --pretty -o "${BASE}.xml"

echo "Converting to CSV..."
hedl to-csv "$INPUT" -o "${BASE}.csv"

echo "Converting to Parquet..."
hedl to-parquet "$INPUT" -o "${BASE}.parquet"

echo "All conversions complete!"
ls -lh "${BASE}".*
```

## AI/ML Workflows

### Example 8: Training Data Preparation

Prepare token-efficient training data for LLMs.

**Original (training_data.json - 450 tokens):**
```json
{
  "examples": [
    {
      "input": "What is the capital of France?",
      "output": "Paris",
      "metadata": {"category": "geography", "difficulty": "easy"}
    },
    {
      "input": "Explain photosynthesis",
      "output": "Photosynthesis is...",
      "metadata": {"category": "science", "difficulty": "medium"}
    }
  ]
}
```

**HEDL (training_data.hedl - 180 tokens, 60% reduction):**
```hedl
%VERSION: 1.0
%STRUCT: Example: [input, output, category, difficulty]
---
examples: @Example
  | What is the capital of France?, Paris, geography, easy
  | Explain photosynthesis, Photosynthesis is..., science, medium
```

**Commands:**
```bash
# Convert JSON training data to HEDL
hedl from-json training_data.json -o training_data.hedl

# Validate
hedl validate training_data.hedl

# Check token savings
hedl stats training_data.hedl

# Use in LLM context (convert back to JSON when needed)
hedl to-json training_data.hedl --pretty | llm-tool process
```

### Example 9: Model Configuration

Store model hyperparameters in a readable, version-controlled format.

**HEDL (model_config.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: Layer: [type, units, activation]
---
model:
  name: text-classifier-v2
  architecture: transformer
  hyperparameters:
    learning_rate: 0.001
    batch_size: 32
    epochs: 10
    dropout: 0.1
    max_seq_length: 512
  layers: @Layer
    | embedding, 768, none
    | transformer, 768, gelu
    | pooling, 768, tanh
    | dense, 2, softmax
  training:
    optimizer: adam
    loss: categorical_crossentropy
    metrics: 3
      accuracy
      precision
      recall
```

**Commands:**
```bash
# Format for readability
hedl format model_config.hedl -o model_config.hedl

# Convert to YAML for Kubernetes ConfigMap
hedl to-yaml model_config.hedl -o model_config.yaml

# Convert to JSON for Python loading
hedl to-json model_config.hedl --pretty -o model_config.json
```

### Example 10: Dataset Metadata

Maintain dataset metadata efficiently.

**HEDL (dataset_metadata.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: Dataset: [id, name, records, size_mb, format]
%STRUCT: Split: [dataset, type, percentage, path]
---
datasets: @Dataset
  | ds1, customer_reviews, 1000000, 2500, parquet
  | ds2, product_catalog, 50000, 125, csv
  | ds3, user_interactions, 5000000, 8000, parquet
  | ds4, embeddings_v1, 100000, 4096, npy
  | ds5, test_set, 10000, 25, json

splits: @Split
  | @Dataset:ds1, train, 80, data/reviews/train.parquet
  | @Dataset:ds1, val, 10, data/reviews/val.parquet
  | @Dataset:ds1, test, 10, data/reviews/test.parquet
  | @Dataset:ds2, train, 100, data/catalog/all.csv
  # ... more splits
```

## Data Processing Pipelines

### Example 11: ETL Pipeline

Extract, Transform, Load pipeline using HEDL as intermediate format.

**Pipeline Script:**
```bash
#!/bin/bash
# etl_pipeline.sh - Data processing pipeline

set -euo pipefail

SOURCE_DIR="source_data"
STAGING_DIR="staging"
OUTPUT_DIR="output"

mkdir -p "$STAGING_DIR" "$OUTPUT_DIR"

# EXTRACT: Multiple CSV files to HEDL
echo "=== EXTRACT ==="
for csv in "$SOURCE_DIR"/*.csv; do
  base=$(basename "$csv" .csv)
  echo "Extracting $csv..."
  hedl from-csv "$csv" --headers -o "$STAGING_DIR/${base}.hedl"
done

# TRANSFORM: Validate and format
echo "=== TRANSFORM ==="
echo "Validating all HEDL files..."
if ! hedl batch-validate "$STAGING_DIR/*.hedl"; then
  echo "Validation failed!" >&2
  exit 1
fi

echo "Formatting files..."
hedl batch-format $STAGING_DIR/*.hedl --output-dir formatted/

echo "Linting for quality issues..."
hedl batch-lint $STAGING_DIR/*.hedl

# LOAD: Convert to Parquet for analytics
echo "=== LOAD ==="
for hedl in "$STAGING_DIR"/*.hedl; do
  base=$(basename "$hedl" .hedl)
  echo "Loading $hedl to Parquet..."
  hedl to-parquet "$hedl" -o "$OUTPUT_DIR/${base}.parquet"
done

echo "=== COMPLETE ==="
echo "Processed $(ls "$OUTPUT_DIR"/*.parquet | wc -l) files"
```

### Example 12: Data Quality Validation

Validate data quality during processing.

**Validation Script:**
```bash
#!/bin/bash
# validate_data.sh

DATA_DIR="data"
REPORT_FILE="validation_report.txt"

echo "Data Quality Report" > "$REPORT_FILE"
echo "Generated: $(date)" >> "$REPORT_FILE"
echo "==================" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Validate all HEDL files
for file in "$DATA_DIR"/*.hedl; do
  echo "Checking $file..." | tee -a "$REPORT_FILE"

  # Syntax validation
  if hedl validate "$file" 2>/dev/null; then
    echo "  ✓ Syntax valid" | tee -a "$REPORT_FILE"
  else
    echo "  ✗ SYNTAX ERROR" | tee -a "$REPORT_FILE"
    hedl validate "$file" 2>&1 | tee -a "$REPORT_FILE"
    continue
  fi

  # Linting
  if hedl lint "$file" 2>&1 | tee -a "$REPORT_FILE" | grep -q "0 issues"; then
    echo "  ✓ No lint issues" | tee -a "$REPORT_FILE"
  else
    echo "  ⚠ Lint warnings found" | tee -a "$REPORT_FILE"
  fi

  # Statistics
  echo "  Statistics:" | tee -a "$REPORT_FILE"
  hedl stats "$file" | grep "HEDL:" | tee -a "$REPORT_FILE"

  echo "" >> "$REPORT_FILE"
done

echo "Report saved to $REPORT_FILE"
```

### Example 13: Incremental Data Updates

Handle incremental data updates efficiently.

**Update Script:**
```bash
#!/bin/bash
# incremental_update.sh

MASTER_FILE="master_data.hedl"
UPDATES_FILE="updates.hedl"
BACKUP_FILE="master_data.backup.hedl"

# Backup current master
cp "$MASTER_FILE" "$BACKUP_FILE"

# Validate updates
if ! hedl validate "$UPDATES_FILE"; then
  echo "Updates file is invalid!" >&2
  exit 1
fi

# Merge: Convert both to JSON, merge, convert back
hedl to-json "$MASTER_FILE" > master.json
hedl to-json "$UPDATES_FILE" > updates.json

# Merge logic (using jq)
jq -s '.[0] * .[1]' master.json updates.json > merged.json

# Convert back to HEDL
hedl from-json merged.json -o "$MASTER_FILE"

# Validate result
if hedl validate "$MASTER_FILE"; then
  echo "Update successful!"
  rm master.json updates.json merged.json
else
  echo "Merge validation failed! Restoring backup..." >&2
  cp "$BACKUP_FILE" "$MASTER_FILE"
  exit 1
fi
```

## Configuration Management

### Example 14: Application Configuration

Use HEDL for application config files.

**HEDL (app_config.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: RateLimits: [endpoint, requests_per_minute]
---
app:
  name: MyApp
  version: 1.2.3
  environment: production

  server:
    host: 0.0.0.0
    port: 8080
    workers: 4
    timeout_seconds: 30

  database:
    type: postgresql
    host: db.example.com
    port: 5432
    name: myapp_prod
    pool_size: 10
    ssl_mode: require

  logging:
    level: info
    format: json
    outputs: 2
      stdout
      /var/log/myapp/app.log

  features:
    new_ui_enabled: true
    beta_features: false
    analytics_enabled: true

  rate_limits: @RateLimits
    | /api/users, 100
    | /api/search, 50
    | /api/admin, 20
```

**Usage:**
```bash
# Validate config
hedl validate app_config.hedl

# Deploy: Convert to JSON for application
hedl to-json app_config.hedl --pretty -o /etc/myapp/config.json

# Deploy: Convert to YAML for Kubernetes
hedl to-yaml app_config.hedl -o k8s/configmap.yaml
```

### Example 15: Multi-Environment Configs

Manage configurations for multiple environments.

**Directory Structure:**
```
configs/
  base.hedl
  dev.hedl
  staging.hedl
  production.hedl
```

**base.hedl:**
```hedl
%VERSION: 1.0
---
app:
  name: MyApp
  version: 1.2.3
  server:
    workers: 4
    timeout_seconds: 30
```

**production.hedl:**
```hedl
%VERSION: 1.0
---
app:
  environment: production
  server:
    host: 0.0.0.0
    port: 443
    ssl_enabled: true
  database:
    host: prod-db.example.com
    port: 5432
```

**Merge Script:**
```bash
#!/bin/bash
# merge_config.sh

ENV="${1:-dev}"

BASE="configs/base.hedl"
ENV_FILE="configs/${ENV}.hedl"
OUTPUT="configs/merged_${ENV}.hedl"

# Convert to JSON
hedl to-json "$BASE" > base.json
hedl to-json "$ENV_FILE" > env.json

# Merge (env overrides base)
jq -s '.[0] * .[1]' base.json env.json > merged.json

# Convert back to HEDL
hedl from-json merged.json -o "$OUTPUT"

# Validate
hedl validate "$OUTPUT"

echo "Config for $ENV environment: $OUTPUT"

# Clean up
rm base.json env.json merged.json
```

## Graph Data

### Example 16: Social Network Graph

Model social network data with relationships.

**HEDL (social_network.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, joined]
%STRUCT: Friendship: [from, to, since]
%STRUCT: Post: [id, author, content, timestamp]
%STRUCT: Like: [post, user]
---
users: @User
  | u1, Alice, 2020-01-15
  | u2, Bob, 2020-02-20
  | u3, Charlie, 2020-03-10
  | u4, Diana, 2020-04-05
  | u5, Eve, 2020-05-12

friendships: @Friendship
  | @User:u1, @User:u2, 2020-02-21
  | @User:u1, @User:u3, 2020-03-15
  | @User:u2, @User:u3, 2020-04-01
  | @User:u2, @User:u4, 2020-05-10
  | @User:u3, @User:u5, 2020-06-20
  | @User:u4, @User:u5, 2020-07-01

posts: @Post
  | p1, @User:u1, Hello world!, 2020-02-22T10:00:00Z
  | p2, @User:u2, Great day today!, 2020-02-23T14:30:00Z
  | p3, @User:u3, Check out this link, 2020-02-24T09:15:00Z

likes: @Like
  | @Post:p1, @User:u2
  | @Post:p1, @User:u3
  | @Post:p2, @User:u1
  | @Post:p2, @User:u3
  | @Post:p3, @User:u1
```

**Export to Neo4j:**

*Note: Cypher export is currently available via the `hedl-neo4j` Rust library or the MCP server, not the command-line tool.*

```rust
// Rust Library Usage
use hedl_neo4j::{to_cypher, ToCypherConfig};

let cypher = to_cypher(&doc, &ToCypherConfig::default())?;
println!("{}", cypher);
```

### Example 17: Knowledge Graph

Represent knowledge graph with entities and relationships.

**HEDL (knowledge_graph.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: Concept: [id, name, type]
%STRUCT: Relationship: [from, to, type]
---
concepts: @Concept
  | c1, Machine Learning, field
  | c2, Neural Networks, technique
  | c3, Deep Learning, subfield
  | c4, Supervised Learning, paradigm
  | c5, Unsupervised Learning, paradigm
  | c6, Classification, task
  | c7, Regression, task
  | c8, Clustering, task
  | c9, Python, language
  | c10, TensorFlow, framework

relationships: @Relationship
  | @Concept:c3, @Concept:c1, subset_of
  | @Concept:c2, @Concept:c3, technique_in
  | @Concept:c4, @Concept:c1, paradigm_of
  | @Concept:c5, @Concept:c1, paradigm_of
  | @Concept:c6, @Concept:c4, task_in
  | @Concept:c7, @Concept:c4, task_in
  | @Concept:c8, @Concept:c5, task_in
  | @Concept:c10, @Concept:c9, implemented_in
  | @Concept:c10, @Concept:c3, used_for
  | @Concept:c1, @Concept:c9, commonly_uses
```

## Advanced Patterns

### Example 18: Tensor Data

Store multi-dimensional tensor data efficiently.

**HEDL (tensor_data.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: Embedding: [id, dimensions, values]
%STRUCT: Matrix: [id, shape, data]
---
embeddings: @Embedding
  | emb1, [768], [0.1, 0.2, 0.3, ..., 0.768]
  | emb2, [768], [0.5, 0.6, 0.7, ..., 0.768]
  | emb3, [768], [0.9, 0.8, 0.7, ..., 0.001]

matrices: @Matrix
  | m1, [3,3], [[1,2,3],[4,5,6],[7,8,9]]
  | m2, [2,4], [[1,2,3,4],[5,6,7,8]]
```

### Example 19: Time Series Data

Efficient storage of time series data.

**HEDL (timeseries.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: Metric: [timestamp, cpu_percent, memory_mb, requests_per_sec]
---
metrics: @Metric
  | 2024-01-01T00:00:00Z, 45.2, 2048, 150
  | 2024-01-01T00:01:00Z, 47.8, 2056, 165
  | 2024-01-01T00:02:00Z, 43.1, 2045, 142
  # ... 1437 more minute-level samples
```

**Benefits:**
- Compact storage (60% less than JSON)
- Easy to parse and validate
- Can convert to Parquet for analytics

### Example 20: Complex Nested Hierarchy

Maximum nesting capabilities.

**HEDL (complex_structure.hedl):**
```hedl
%VERSION: 1.0
%STRUCT: Division: [id, name]
%STRUCT: Region: [id, name]
%STRUCT: Office: [region, city, employees]
---
organization:
  name: TechCorp
  divisions: @Division
    | d1, North America
    | d2, Europe
  division_details:
    division_id: @Division:d1
    regions: @Region
      | r1, West Coast
      | r2, East Coast
      | r3, Central
    offices: @Office
      | @Region:r1, San Francisco, 250
      | @Region:r1, Seattle, 180
      | @Region:r2, New York, 400
      | @Region:r2, Boston, 120
      | @Region:r3, Chicago, 200
```

---

## Summary of Use Cases

| Use Case | Benefits | Best Format |
|----------|----------|-------------|
| API Responses | Token efficiency, validation | HEDL |
| Training Data | 60% fewer tokens, structure | HEDL |
| Configuration | Readability, version control | HEDL or YAML |
| Analytics | Fast queries, compression | Parquet |
| Spreadsheets | Compatibility, simplicity | CSV |
| Graph Databases | Relationships, Cypher export | HEDL + Neo4j |
| Data Pipelines | Validation, multi-format | HEDL (hub) |

---

**Ready for more?** Check out the [CLI Guide](cli-guide.md) for all available commands or [Troubleshooting](troubleshooting.md) for help with common issues.
