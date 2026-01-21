// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! YAML Alias Resolution Optimization Benchmarks
//!
//! This benchmark suite validates the performance improvements from the
//! alias resolution optimization (Task 92). It tests:
//! - Varying numbers of anchors and aliases
//! - Realistic K8s ConfigMap patterns
//! - Deeply nested alias structures
//! - Memory efficiency of shared structures

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use hedl_yaml::{from_yaml, FromYamlConfig};

/// Generates a YAML document with specified anchors and alias references
fn generate_alias_heavy_yaml(anchor_count: usize, alias_refs_per_anchor: usize) -> String {
    let mut yaml = String::new();
    yaml.push_str("---\n");

    // Generate anchor definitions
    for i in 0..anchor_count {
        yaml.push_str(&format!("anchor{}: &anchor{}\n", i, i));
        yaml.push_str(&format!("  data: value_{}\n", i));
        yaml.push_str(&format!("  index: {}\n", i));
        yaml.push_str("  nested:\n");
        yaml.push_str(&format!("    value: {}\n", i * 100));
        yaml.push_str("    flag: true\n");
    }

    // Generate alias references
    yaml.push_str("references:\n");
    for i in 0..anchor_count {
        for j in 0..alias_refs_per_anchor {
            yaml.push_str(&format!("  ref_{}_{}: *anchor{}\n", i, j, i));
        }
    }

    yaml
}

/// Benchmark: Parse documents with varying anchor counts
fn bench_alias_resolution_varying_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("yaml_alias_resolution");

    for anchor_count in [10, 50, 100, 500, 1000].iter() {
        let yaml = generate_alias_heavy_yaml(*anchor_count, 10);
        let bytes = yaml.len();

        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("anchors", anchor_count),
            &yaml,
            |b, yaml| {
                b.iter(|| {
                    let config = FromYamlConfig::default();
                    from_yaml(black_box(yaml), &config).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Realistic Kubernetes ConfigMap pattern with shared base configuration
fn bench_k8s_configmap_pattern(c: &mut Criterion) {
    let yaml = r#"---
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  base_config: &base
    database:
      host: localhost
      port: 5432
      ssl: true
      pool_size: 20
      timeout: 30000
    cache:
      ttl: 3600
      max_size: 100MB
      eviction_policy: lru
    logging:
      level: info
      format: json
      rotation: daily
      max_size: 1GB
    metrics:
      enabled: true
      port: 9090
      path: /metrics

  production:
    <<: *base
    database:
      host: prod.db.example.com
      pool_size: 100
    logging:
      level: warn
    metrics:
      enabled: true

  staging:
    <<: *base
    database:
      host: staging.db.example.com
      pool_size: 50
    logging:
      level: debug

  development:
    <<: *base
    database:
      host: localhost
      pool_size: 10
    logging:
      level: trace

  testing:
    <<: *base
    database:
      host: test.db.example.com
      pool_size: 20
    logging:
      level: debug
"#;

    c.bench_function("k8s_configmap_realistic", |b| {
        b.iter(|| {
            let config = FromYamlConfig::default();
            from_yaml(black_box(yaml), &config).unwrap()
        });
    });
}

/// Benchmark: Deeply nested alias structures
fn bench_deeply_nested_aliases(c: &mut Criterion) {
    let yaml = r#"---
level1: &l1
  data: base
  value: 100
  nested: &l2
    data: middle
    value: 200
    deep: &l3
      data: deep
      value: 300
      deeper: &l4
        data: deeper
        value: 400
        deepest: &l5
          data: deepest
          value: 500

ref_l1_1: *l1
ref_l1_2: *l1
ref_l1_3: *l1
ref_l2_1: *l2
ref_l2_2: *l2
ref_l2_3: *l2
ref_l3_1: *l3
ref_l3_2: *l3
ref_l3_3: *l3
ref_l4_1: *l4
ref_l4_2: *l4
ref_l4_3: *l4
ref_l5_1: *l5
ref_l5_2: *l5
ref_l5_3: *l5
"#;

    c.bench_function("deeply_nested_aliases", |b| {
        b.iter(|| {
            let config = FromYamlConfig::default();
            from_yaml(black_box(yaml), &config).unwrap()
        });
    });
}

/// Benchmark: Many references to same large structure
fn bench_many_refs_to_large_structure(c: &mut Criterion) {
    let mut yaml = String::from("---\nshared: &shared\n");

    // Create a large shared structure (1KB of data)
    for i in 0..20 {
        yaml.push_str(&format!("  field_{}: value_{}\n", i, i));
        yaml.push_str(&format!("  data_{}: {}\n", i, "x".repeat(30)));
    }

    yaml.push_str("\nrefs:\n");
    // Reference it 100 times
    for i in 0..100 {
        yaml.push_str(&format!("  ref{}: *shared\n", i));
    }

    c.bench_function("many_refs_large_structure", |b| {
        b.iter(|| {
            let config = FromYamlConfig::default();
            from_yaml(black_box(&yaml), &config).unwrap()
        });
    });
}

/// Benchmark: Docker Compose style with service templates
fn bench_docker_compose_pattern(c: &mut Criterion) {
    let yaml = r#"---
x-service-template: &service-template
  restart: unless-stopped
  networks:
    - app-network
  logging:
    driver: json-file
    options:
      max-size: "10m"
      max-file: "3"
  deploy:
    resources:
      limits:
        cpus: '0.50'
        memory: 512M
      reservations:
        cpus: '0.25'
        memory: 256M

services:
  web:
    <<: *service-template
    image: nginx:latest
    ports:
      - "80:80"
      - "443:443"

  api:
    <<: *service-template
    image: api:v1.0
    ports:
      - "8080:8080"

  worker:
    <<: *service-template
    image: worker:v1.0

  cache:
    <<: *service-template
    image: redis:alpine
    ports:
      - "6379:6379"

  db:
    <<: *service-template
    image: postgres:14
    ports:
      - "5432:5432"
"#;

    c.bench_function("docker_compose_pattern", |b| {
        b.iter(|| {
            let config = FromYamlConfig::default();
            from_yaml(black_box(yaml), &config).unwrap()
        });
    });
}

/// Benchmark: GitHub Actions workflow with job templates
fn bench_github_actions_pattern(c: &mut Criterion) {
    let yaml = r#"---
x-test-job: &test-job
  runs-on: ubuntu-latest
  timeout-minutes: 30
  env:
    RUST_BACKTRACE: 1
    CARGO_INCREMENTAL: 0
  steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable

jobs:
  test-linux:
    <<: *test-job
    name: Test on Linux

  test-macos:
    <<: *test-job
    name: Test on macOS
    runs-on: macos-latest

  test-windows:
    <<: *test-job
    name: Test on Windows
    runs-on: windows-latest

  test-nightly:
    <<: *test-job
    name: Test with nightly
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: nightly
"#;

    c.bench_function("github_actions_pattern", |b| {
        b.iter(|| {
            let config = FromYamlConfig::default();
            from_yaml(black_box(yaml), &config).unwrap()
        });
    });
}

/// Benchmark: Zero anchors (baseline - should show minimal overhead)
fn bench_no_anchors_baseline(c: &mut Criterion) {
    let yaml = r#"---
user1:
  name: Alice
  age: 30
  active: true

user2:
  name: Bob
  age: 25
  active: false

user3:
  name: Charlie
  age: 35
  active: true

user4:
  name: Diana
  age: 28
  active: true

user5:
  name: Eve
  age: 32
  active: false
"#;

    c.bench_function("no_anchors_baseline", |b| {
        b.iter(|| {
            let config = FromYamlConfig::default();
            from_yaml(black_box(yaml), &config).unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_alias_resolution_varying_counts,
    bench_k8s_configmap_pattern,
    bench_deeply_nested_aliases,
    bench_many_refs_to_large_structure,
    bench_docker_compose_pattern,
    bench_github_actions_pattern,
    bench_no_anchors_baseline,
);
criterion_main!(benches);
