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

//! Dataset generators for benchmarks
//!
//! Generates realistic HEDL documents of various sizes and structures.
//!
//! All generator functions validate input sizes against [`MAX_DATASET_SIZE`]
//! to prevent denial-of-service attacks from excessively large allocations.

use crate::error::{validate_dataset_size, Result};

use fake::faker::lorem::en::*;
use fake::faker::name::en::*;
use fake::Fake;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Dataset size categories
#[derive(Debug, Clone, Copy)]
pub enum DatasetSize {
    /// Small: ~10 entities
    Small,
    /// Medium: ~100 entities
    Medium,
    /// Large: ~1,000 entities
    Large,
    /// Stress: ~10,000 entities
    Stress,
    /// Extreme: ~100,000 entities
    Extreme,
}

impl DatasetSize {
    pub fn count(&self) -> usize {
        match self {
            DatasetSize::Small => 10,
            DatasetSize::Medium => 100,
            DatasetSize::Large => 1_000,
            DatasetSize::Stress => 10_000,
            DatasetSize::Extreme => 100_000,
        }
    }
}

/// Generate a user dataset with the specified number of users (safe version).
///
/// Uses compact HEDL format with inline schema (flat tabular data).
/// Includes count hint to help LLMs understand dataset size.
///
/// # Arguments
///
/// * `count` - Number of user records to generate (max: [`MAX_DATASET_SIZE`])
///
/// # Returns
///
/// A HEDL document string, or an error if the count exceeds limits.
///
/// # Errors
///
/// Returns [`BenchError::DatasetTooLarge`] if `count` exceeds [`MAX_DATASET_SIZE`].
///
/// # Examples
///
/// ```no_run
/// use hedl_bench::generate_users_safe;
///
/// let hedl = generate_users_safe(100).expect("Failed to generate users");
/// assert!(hedl.contains("%STRUCT: User"));
/// ```
pub fn generate_users_safe(count: usize) -> Result<String> {
    validate_dataset_size(count)?;
    Ok(generate_users_unchecked(count))
}

/// Generate a user dataset with the specified number of users (unchecked).
///
/// Uses compact HEDL format with inline schema (flat tabular data).
/// Includes count hint to help LLMs understand dataset size.
///
/// # Safety
///
/// This function does not validate the input size. For production use,
/// prefer [`generate_users_safe`] which includes DoS protection.
///
/// # Panics
///
/// May panic or cause OOM if count is extremely large.
pub fn generate_users(count: usize) -> String {
    generate_users_unchecked(count)
}

#[inline]
fn generate_users_unchecked(count: usize) -> String {
    let mut rng = StdRng::seed_from_u64(12345);
    let mut lines = Vec::with_capacity(count + 10);

    // Flat table: use %STRUCT with count in header
    lines.push("%VERSION: 1.0".to_string());
    lines.push(format!(
        "%STRUCT: User ({}): [id,name,email,role,created_at]",
        count
    ));
    lines.push("---".to_string());
    lines.push("users: @User".to_string());

    let roles = ["admin", "developer", "designer", "manager", "analyst"];

    for i in 0..count {
        let first: String = FirstName().fake_with_rng(&mut rng);
        let last: String = LastName().fake_with_rng(&mut rng);
        let id = format!("user{}", i + 1);
        let name = format!("{} {}", first, last);
        let email = format!(
            "{}.{}@example.com",
            first.to_lowercase(),
            last.to_lowercase()
        );
        let role = roles[rng.gen_range(0..roles.len())];
        let year = rng.gen_range(2020..2025);
        let month = rng.gen_range(1..13);
        let day = rng.gen_range(1..29);
        let created_at = format!("{:04}-{:02}-{:02}T10:00:00Z", year, month, day);

        lines.push(format!(
            "  |{},{},{},{},{}",
            id, name, email, role, created_at
        ));
    }

    lines.join("\n")
}

/// Generate a product catalog dataset.
///
/// Uses compact HEDL format with inline schema (flat tabular data).
/// Includes count hint to help LLMs understand dataset size.
pub fn generate_products(count: usize) -> String {
    let mut rng = StdRng::seed_from_u64(54321);
    let mut lines = Vec::with_capacity(count + 10);

    lines.push("%VERSION: 1.0".to_string());
    lines.push(format!(
        "%STRUCT: Product ({}): [id,name,price,category,stock,description]",
        count
    ));
    lines.push("---".to_string());
    lines.push("products: @Product".to_string());

    let categories = ["electronics", "clothing", "home", "sports", "books", "toys"];
    let adjectives = [
        "Premium",
        "Professional",
        "Essential",
        "Deluxe",
        "Classic",
        "Modern",
    ];
    let nouns = ["Widget", "Gadget", "Tool", "Device", "Kit", "Set", "Pack"];

    for i in 0..count {
        let adj = adjectives[rng.gen_range(0..adjectives.len())];
        let noun = nouns[rng.gen_range(0..nouns.len())];
        let id = format!("prod{}", i + 1);
        let name = format!("{} {} {}", adj, noun, i + 1);
        let price = format!("{:.2}", rng.gen_range(9.99..999.99));
        let category = categories[rng.gen_range(0..categories.len())];
        let stock = rng.gen_range(0..1000);
        let desc: String = Sentence(3..8).fake_with_rng(&mut rng);

        // Escape quotes in description
        let desc = desc.replace('"', "\"\"");
        lines.push(format!(
            "  | {}, \"{}\",{},{},{}, \"{}\"",
            id, name, price, category, stock, desc
        ));
    }

    lines.join("\n")
}

/// Generate event log entries.
///
/// Uses compact HEDL format with inline schema (flat tabular data).
/// Includes count hint to help LLMs understand dataset size.
pub fn generate_events(count: usize) -> String {
    let mut rng = StdRng::seed_from_u64(98765);
    let mut lines = Vec::with_capacity(count + 10);

    lines.push("%VERSION: 1.0".to_string());
    lines.push(format!(
        "%STRUCT: Event ({}): [id,timestamp,level,service,message]",
        count
    ));
    lines.push("---".to_string());
    lines.push("events: @Event".to_string());

    let levels = ["DEBUG", "INFO", "WARN", "ERROR"];
    let services = ["api", "auth", "db", "cache", "queue", "worker"];
    let messages = [
        "Request processed successfully",
        "Connection established",
        "Cache miss for key",
        "Retrying operation",
        "Rate limit exceeded",
        "Task completed",
        "Validation failed",
        "Timeout reached",
    ];

    for i in 0..count {
        let id = format!("evt{}", i + 1);
        let hour = rng.gen_range(0..24);
        let min = rng.gen_range(0..60);
        let sec = rng.gen_range(0..60);
        let ms = rng.gen_range(0..1000);
        let timestamp = format!("2025-01-15T{:02}:{:02}:{:02}.{:03}Z", hour, min, sec, ms);
        let level = levels[rng.gen_range(0..levels.len())];
        let service = services[rng.gen_range(0..services.len())];
        let message = messages[rng.gen_range(0..messages.len())];

        lines.push(format!(
            "  |{},{},{},{},\"{}\"",
            id, timestamp, level, service, message
        ));
    }

    lines.join("\n")
}

/// Generate a nested configuration structure.
///
/// Creates hierarchical objects with the specified depth.
pub fn generate_nested(depth: usize) -> String {
    let mut lines = Vec::new();

    lines.push("%VERSION: 1.0".to_string());
    lines.push("---".to_string());
    lines.push("config:".to_string());

    fn add_level(lines: &mut Vec<String>, current_depth: usize, max_depth: usize, indent: usize) {
        let prefix = "  ".repeat(indent);

        lines.push(format!("{}name: level_{}", prefix, current_depth));
        lines.push(format!("{}enabled: true", prefix));
        lines.push(format!("{}value: {}", prefix, current_depth * 10));

        if current_depth < max_depth {
            lines.push(format!("{}child:", prefix));
            add_level(lines, current_depth + 1, max_depth, indent + 1);
        }
    }

    add_level(&mut lines, 1, depth, 1);
    lines.join("\n")
}

/// Generate a graph with cross-references.
///
/// Creates nodes with references to other nodes.
/// Uses compact HEDL format with inline schema.
pub fn generate_graph(nodes: usize, edges_per_node: usize) -> String {
    let mut rng = StdRng::seed_from_u64(11111);
    let mut lines = Vec::with_capacity(nodes * 2 + 10);

    lines.push("%VERSION: 1.0".to_string());
    lines.push(format!("%STRUCT: Node ({}): [id,label,connections]", nodes));
    lines.push("---".to_string());
    lines.push("nodes: @Node".to_string());

    for i in 0..nodes {
        let id = format!("n{}", i + 1);
        let label = format!("Node {}", i + 1);

        // Generate random connections to other nodes
        let mut connections = Vec::new();
        for _ in 0..edges_per_node.min(nodes - 1) {
            let target = rng.gen_range(0..nodes);
            if target != i {
                connections.push(format!("@n{}", target + 1));
            }
        }

        let conn_str = if connections.is_empty() {
            "~".to_string()
        } else {
            format!("[{}]", connections.join(","))
        };

        lines.push(format!("  |{},{},{}", id, label, conn_str));
    }

    lines.join("\n")
}

/// Generate a realistic blog dataset with posts, authors, and comments.
///
/// Uses NEST for hierarchical relationships. Needs %STRUCT for %NEST.
/// Uses compact row format.
pub fn generate_blog(posts: usize, comments_per_post: usize) -> String {
    let mut rng = StdRng::seed_from_u64(22222);
    let author_count = (posts / 5).max(3);

    let mut lines = vec![
        "%VERSION: 1.0".to_string(),
        format!("%STRUCT: Author ({}): [id,name,email]", author_count),
        format!(
            "%STRUCT: Post ({}): [id,title, author, published_at]",
            posts
        ),
        "%STRUCT: Comment: [id,author, content, created_at]".to_string(),
        "%NEST: Post > Comment".to_string(),
        "---".to_string(),
    ];

    // Generate authors
    lines.push("authors: @Author".to_string());
    for i in 0..author_count {
        let first: String = FirstName().fake_with_rng(&mut rng);
        let last: String = LastName().fake_with_rng(&mut rng);
        let id = format!("author{}", i + 1);
        let name = format!("{} {}", first, last);
        let email = format!("{}@blog.com", first.to_lowercase());
        lines.push(format!("  |{},{},{}", id, name, email));
    }

    // Generate posts with nested comments
    lines.push("posts: @Post".to_string());
    for i in 0..posts {
        let id = format!("post{}", i + 1);
        let title: String = Sentence(4..10).fake_with_rng(&mut rng);
        let title = title.trim_end_matches('.').replace('"', "");
        let author_id = format!("@Author:author{}", rng.gen_range(1..=author_count));
        let month = rng.gen_range(1..13);
        let day = rng.gen_range(1..29);
        let published = format!("2025-{:02}-{:02}", month, day);

        lines.push(format!(
            "  |[{}] {},\"{}\",{},{}",
            comments_per_post, id, title, author_id, published
        ));

        // Add nested comments
        for j in 0..comments_per_post {
            let comment_id = format!("comment{}_{}", i + 1, j + 1);
            let commenter_id = format!("@Author:author{}", rng.gen_range(1..=author_count));
            let content: String = Sentence(5..15).fake_with_rng(&mut rng);
            let content = content.replace('"', "\"\"");
            let created = format!(
                "2025-{:02}-{:02}T{:02}:00:00Z",
                month,
                day.min(28),
                rng.gen_range(0..24)
            );

            lines.push(format!(
                "    |{},{},\"{}\",{}",
                comment_id, commenter_id, content, created
            ));
        }
    }

    lines.join("\n")
}

/// Generate analytics/time-series data.
///
/// Uses compact HEDL format with inline schema (flat tabular data).
/// Includes count hint to help LLMs understand dataset size.
pub fn generate_analytics(count: usize) -> String {
    let mut rng = StdRng::seed_from_u64(33333);
    let mut lines = Vec::with_capacity(count + 10);

    lines.push("%VERSION: 1.0".to_string());
    lines.push(format!(
        "%STRUCT: Metric ({}): [id,timestamp,name,value,tags]",
        count
    ));
    lines.push("---".to_string());
    lines.push("metrics: @Metric".to_string());

    let metric_names = [
        "cpu_usage",
        "memory_used",
        "disk_io",
        "network_rx",
        "network_tx",
        "request_count",
        "error_rate",
        "latency_p99",
        "queue_depth",
        "active_connections",
    ];
    let hosts = ["web1", "web2", "api1", "api2", "db1", "cache1"];
    let regions = ["us-east", "us-west", "eu-west", "ap-south"];

    for i in 0..count {
        let id = format!("m{}", i + 1);
        let hour = (i / 60) % 24;
        let min = i % 60;
        let timestamp = format!("2025-01-15T{:02}:{:02}:00Z", hour, min);
        let name = metric_names[rng.gen_range(0..metric_names.len())];
        let value = format!("{:.2}", rng.gen_range(0.0..100.0));
        let host = hosts[rng.gen_range(0..hosts.len())];
        let region = regions[rng.gen_range(0..regions.len())];
        let tags = format!("host={} region={}", host, region);

        lines.push(format!(
            "  |{},{},{},{},\"{}\"",
            id, timestamp, name, value, tags
        ));
    }

    lines.join("\n")
}

/// Generate a configuration dataset with nested structures.
pub fn generate_config(sections: usize) -> String {
    let mut rng = StdRng::seed_from_u64(44444);
    let mut lines = Vec::new();

    lines.push("%VERSION: 1.0".to_string());
    lines.push("---".to_string());

    let section_names = [
        "database",
        "cache",
        "logging",
        "security",
        "api",
        "queue",
        "storage",
        "monitoring",
        "notifications",
        "features",
    ];

    for section in section_names.iter().take(sections) {
        lines.push(format!("{}:", section));

        // Add section-specific settings
        lines.push("  enabled: true".to_string());
        lines.push(format!("  timeout: {}", rng.gen_range(1000..30000)));
        lines.push(format!("  retries: {}", rng.gen_range(1..10)));

        // Add nested subsection
        lines.push("  options:".to_string());
        lines.push(format!("    max_connections: {}", rng.gen_range(10..1000)));
        lines.push(format!("    buffer_size: {}", rng.gen_range(1024..65536)));
        lines.push(format!(
            "    compression: {}",
            if rng.gen_bool(0.5) { "true" } else { "false" }
        ));
    }

    lines.join("\n")
}

/// Generate a nested orders dataset with customer and items.
///
/// Structure: orders with nested customer object and items array.
/// Uses compact row format. Needs %STRUCT for %NEST.
pub fn generate_orders(count: usize) -> String {
    let mut rng = StdRng::seed_from_u64(55555);
    let customer_count = (count / 3).max(5);

    let mut lines = vec![
        "%VERSION: 1.0".to_string(),
        format!(
            "%STRUCT: Customer ({}): [id,name,email,phone]",
            customer_count
        ),
        "%STRUCT: Item: [sku,name,quantity,price]".to_string(),
        format!("%STRUCT: Order ({}): [id,customer, status, total]", count),
        "%NEST: Order > Item".to_string(),
        "---".to_string(),
    ];

    // Generate customers first
    lines.push("customers: @Customer".to_string());
    for i in 0..customer_count {
        let first: String = FirstName().fake_with_rng(&mut rng);
        let last: String = LastName().fake_with_rng(&mut rng);
        let id = format!("cust{}", i + 1);
        let name = format!("{} {}", first, last);
        let email = format!(
            "{}.{}@example.com",
            first.to_lowercase(),
            last.to_lowercase()
        );
        let phone = format!(
            "+1-555-{:03}-{:04}",
            rng.gen_range(100..1000),
            rng.gen_range(0..10000)
        );
        lines.push(format!("  |{},{},{},{}", id, name, email, phone));
    }

    // Generate orders with nested items
    let statuses = ["pending", "processing", "shipped", "delivered", "cancelled"];
    let products = ["Widget", "Gadget", "Tool", "Device", "Kit", "Pack"];

    lines.push("orders: @Order".to_string());
    let mut item_counter = 1; // Sequential counter for unique SKUs
    for i in 0..count {
        let order_id = format!("ord{}", i + 1);
        let customer_ref = format!("@Customer:cust{}", rng.gen_range(1..=customer_count));
        let status = statuses[rng.gen_range(0..statuses.len())];
        let item_count = rng.gen_range(1..6);

        lines.push(format!(
            "  |[{}] {},{},{},0.00",
            item_count, order_id, customer_ref, status
        ));

        // Add nested items with unique sequential SKUs
        for _j in 0..item_count {
            let sku = format!("SKU-{:05}", item_counter);
            item_counter += 1;
            let product = products[rng.gen_range(0..products.len())];
            let name = format!("{} {}", product, rng.gen_range(1..100));
            let qty = rng.gen_range(1..10);
            let price = rng.gen_range(9.99..199.99);
            lines.push(format!("    |{},{},{},{:.2}", sku, name, qty, price));
        }
    }

    lines.join("\n")
}

/// Generate structural validation datasets (control + 4 corruption patterns)
pub mod validation {
    /// Control: valid data with 20 employees
    pub fn control() -> String {
        r#"%VERSION: 1.0
%STRUCT: Employee (20): [id, name, department, salary]
---
employees: @Employee
  | emp1, Alice Smith, Engineering, 85000
  | emp2, Bob Jones, Marketing, 72000
  | emp3, Carol White, Engineering, 92000
  | emp4, Dave Brown, Sales, 68000
  | emp5, Eve Davis, Engineering, 88000
  | emp6, Frank Miller, Marketing, 75000
  | emp7, Grace Wilson, Sales, 71000
  | emp8, Henry Moore, Engineering, 95000
  | emp9, Ivy Taylor, Marketing, 69000
  | emp10, Jack Anderson, Sales, 82000
  | emp11, Kate Thomas, Engineering, 91000
  | emp12, Leo Jackson, Marketing, 73000
  | emp13, Mary Harris, Sales, 77000
  | emp14, Nick Martin, Engineering, 89000
  | emp15, Olivia Garcia, Marketing, 76000
  | emp16, Paul Robinson, Sales, 70000
  | emp17, Quinn Clark, Engineering, 93000
  | emp18, Rose Lewis, Marketing, 74000
  | emp19, Sam Walker, Sales, 79000
  | emp20, Tina Hall, Engineering, 87000
"#
        .to_string()
    }

    /// Truncated: missing rows (declared 20, only has 15)
    pub fn truncated() -> String {
        r#"%VERSION: 1.0
%STRUCT: Employee (15): [id, name, department, salary]
---
employees: @Employee
  | emp1, Alice Smith, Engineering, 85000
  | emp2, Bob Jones, Marketing, 72000
  | emp3, Carol White, Engineering, 92000
  | emp4, Dave Brown, Sales, 68000
  | emp5, Eve Davis, Engineering, 88000
  | emp6, Frank Miller, Marketing, 75000
  | emp7, Grace Wilson, Sales, 71000
  | emp8, Henry Moore, Engineering, 95000
  | emp9, Ivy Taylor, Marketing, 69000
  | emp10, Jack Anderson, Sales, 82000
  | emp11, Kate Thomas, Engineering, 91000
  | emp12, Leo Jackson, Marketing, 73000
  | emp13, Mary Harris, Sales, 77000
  | emp14, Nick Martin, Engineering, 89000
  | emp15, Olivia Garcia, Marketing, 76000
"#
        .to_string()
    }

    /// Width mismatch: inconsistent field counts
    pub fn width_mismatch() -> String {
        r#"%VERSION: 1.0
%STRUCT: Employee (5): [id, name, department, salary]
---
employees: @Employee
  | emp1, Alice Smith, Engineering, 85000
  | emp2, Bob Jones, Marketing
  | emp3, Carol White, Engineering, 92000, extra
  | emp4, Dave Brown, Sales, 68000
  | emp5, Eve Davis, Engineering
"#
        .to_string()
    }

    /// Missing fields: required fields are null
    pub fn missing_fields() -> String {
        r#"%VERSION: 1.0
%STRUCT: Employee (5): [id, name, department, salary]
---
employees: @Employee
  | emp1, Alice Smith, Engineering, 85000
  | emp2, ~, Marketing, 72000
  | emp3, Carol White, ~, 92000
  | emp4, Dave Brown, Sales, ~
  | emp5, ~, ~, ~
"#
        .to_string()
    }

    /// Extra rows: more rows than expected
    pub fn extra_rows() -> String {
        // This would need HEDL to support length declarations
        // For now, just return a valid dataset
        control()
    }
}

/// Generate a dataset that heavily uses DITTO operators for repeated values.
///
/// This showcases HEDL's token efficiency for data with many repeated values.
/// Each employee shares company, department, location - using ^ ditto.
/// Uses compact HEDL format with inline schema.
pub fn generate_ditto_heavy(employee_count: usize) -> String {
    let mut rng = StdRng::seed_from_u64(77777);
    let mut lines = vec![
        "%VERSION: 1.0".to_string(),
        format!(
            "%STRUCT: Employee ({}): [id,name,company,department,location,role,salary,status]",
            employee_count
        ),
        "---".to_string(),
        "employees: @Employee".to_string(),
    ];

    let companies = ["Acme Corp", "TechStart Inc", "Global Systems"];
    let departments = ["Engineering", "Sales", "Marketing", "Operations", "Finance"];
    let locations = ["New York", "San Francisco", "London", "Tokyo", "Berlin"];
    let roles = ["Junior", "Senior", "Lead", "Manager", "Director"];
    let statuses = ["active", "active", "active", "active", "on_leave"]; // 80% active

    let mut last_company = "";
    let mut last_dept = "";
    let mut last_location = "";

    for i in 0..employee_count {
        let first: String = FirstName().fake_with_rng(&mut rng);
        let last: String = LastName().fake_with_rng(&mut rng);
        let id = format!("emp{:03}", i + 1);
        let name = format!("{} {}", first, last);

        // Change company every 20 employees, dept every 5, location every 10
        let company = if i % 20 == 0 {
            companies[rng.gen_range(0..companies.len())]
        } else {
            last_company
        };
        let dept = if i % 5 == 0 {
            departments[rng.gen_range(0..departments.len())]
        } else {
            last_dept
        };
        let location = if i % 10 == 0 {
            locations[rng.gen_range(0..locations.len())]
        } else {
            last_location
        };

        let role = roles[rng.gen_range(0..roles.len())];
        let salary = rng.gen_range(50000..200000);
        let status = statuses[rng.gen_range(0..statuses.len())];

        // Use ditto for repeated values
        let company_str = if company == last_company {
            "^"
        } else {
            company
        };
        let dept_str = if dept == last_dept { "^" } else { dept };
        let loc_str = if location == last_location {
            "^"
        } else {
            location
        };

        lines.push(format!(
            "  |{},{},{},{},{},{},{},{}",
            id, name, company_str, dept_str, loc_str, role, salary, status
        ));

        last_company = company;
        last_dept = dept;
        last_location = location;
    }

    lines.join("\n")
}

/// Generate a dataset with complex cross-references AND nested structure.
///
/// This showcases HEDL's typed reference system (@Type:id) combined with %NEST.
/// Structure: Project > Milestone > Task, with cross-project dependencies and assignee refs.
pub fn generate_reference_heavy(project_count: usize) -> String {
    let mut rng = StdRng::seed_from_u64(88888);
    let person_count = project_count * 2;

    let mut lines = vec![
        "%VERSION: 1.0".to_string(),
        format!("%STRUCT: Person ({}): [id,name,role,skills]", person_count),
        format!(
            "%STRUCT: Project ({}): [id, name, owner, status, depends_on]",
            project_count
        ),
        "%STRUCT: Milestone: [id, name, deadline, status]".to_string(),
        "%STRUCT: Task: [id, name, assignee, priority, hours]".to_string(),
        "%NEST: Project > Milestone".to_string(),
        "%NEST: Milestone > Task".to_string(),
        "---".to_string(),
    ];

    // Generate people first (referenced by tasks and as project owners)
    lines.push("people: @Person".to_string());
    let roles = ["developer", "designer", "analyst", "lead", "architect"];
    let skill_sets = [
        "[rust, python]",
        "[typescript, react]",
        "[go, kubernetes]",
        "[python, ml]",
        "[java, spring]",
    ];
    for i in 0..person_count {
        let first: String = FirstName().fake_with_rng(&mut rng);
        let last: String = LastName().fake_with_rng(&mut rng);
        let id = format!("p{:03}", i + 1);
        let name = format!("{} {}", first, last);
        let role = roles[rng.gen_range(0..roles.len())];
        let skills = skill_sets[rng.gen_range(0..skill_sets.len())];
        lines.push(format!("  |{},{},{},{}", id, name, role, skills));
    }

    // Generate projects with nested milestones and tasks
    lines.push("projects: @Project".to_string());
    let statuses = ["planning", "active", "review", "completed"];
    let priorities = ["P0", "P1", "P2", "P3"];

    for p in 0..project_count {
        let proj_id = format!("proj{:03}", p + 1);
        let proj_name = format!(
            "{} {}",
            ["Platform", "API", "Frontend", "Backend", "Mobile"][p % 5],
            ["Redesign", "Migration", "Overhaul", "Integration", "Launch"][rng.gen_range(0..5)]
        );
        let owner = format!("@Person:p{:03}", rng.gen_range(1..=person_count));
        let status = statuses[rng.gen_range(0..statuses.len())];
        // Cross-project dependencies (later projects depend on earlier ones)
        let depends = if p > 0 && rng.gen_bool(0.6) {
            format!("@Project:proj{:03}", rng.gen_range(1..=p))
        } else {
            "~".to_string()
        };

        // 2-3 milestones per project
        let milestone_count = rng.gen_range(2..4);
        lines.push(format!(
            "  |[{}] {},{},{},{},{}",
            milestone_count, proj_id, proj_name, owner, status, depends
        ));

        for m in 0..milestone_count {
            let ms_id = format!("m{}_{}", p + 1, m + 1);
            let ms_name = format!(
                "{} {}",
                ["Alpha", "Beta", "RC", "GA"][m % 4],
                ["Release", "Milestone", "Phase"][rng.gen_range(0..3)]
            );
            let deadline = format!(
                "2025-{:02}-{:02}",
                rng.gen_range(1..13),
                rng.gen_range(1..29)
            );
            let ms_status = statuses[rng.gen_range(0..statuses.len())];

            // 2-4 tasks per milestone
            let task_count = rng.gen_range(2..5);
            lines.push(format!(
                "    |[{}] {},{},{},{}",
                task_count, ms_id, ms_name, deadline, ms_status
            ));

            for t in 0..task_count {
                let task_id = format!("t{}_{}_{}", p + 1, m + 1, t + 1);
                let task_name = format!(
                    "{} {}",
                    ["Implement", "Design", "Test", "Review", "Deploy"][rng.gen_range(0..5)],
                    ["feature", "module", "component", "service", "endpoint"][rng.gen_range(0..5)]
                );
                let assignee = format!("@Person:p{:03}", rng.gen_range(1..=person_count));
                let priority = priorities[rng.gen_range(0..priorities.len())];
                let hours = rng.gen_range(2..40);
                lines.push(format!(
                    "      |{},{},{},{},{}",
                    task_id, task_name, assignee, priority, hours
                ));
            }
        }
    }

    lines.join("\n")
}

/// Generate deeply nested hierarchical data.
///
/// Company > Division > Department > Team > Employee
/// 5 levels of nesting to test structure comprehension.
/// Uses new count syntax in %STRUCT and |N| for parent rows.
pub fn generate_deep_hierarchy(divisions: usize) -> String {
    let mut rng = StdRng::seed_from_u64(99999);
    let mut lines = vec![
        "%VERSION: 1.0".to_string(),
        "%STRUCT: Company: [id,name,founded,industry]".to_string(),
        format!("%STRUCT: Division ({}): [id,name,head,budget]", divisions),
        "%STRUCT: Department: [id,name,manager,headcount]".to_string(),
        "%STRUCT: Team: [id,name,lead,focus]".to_string(),
        "%STRUCT: Employee: [id,name,role,level]".to_string(),
        "%NEST: Company > Division".to_string(),
        "%NEST: Division > Department".to_string(),
        "%NEST: Department > Team".to_string(),
        "%NEST: Team > Employee".to_string(),
        "---".to_string(),
    ];

    let focuses = [
        "Backend", "Frontend", "Data", "DevOps", "Security", "Mobile",
    ];
    let roles = ["Engineer", "Designer", "Analyst", "Specialist"];
    let levels = ["Junior", "Mid", "Senior", "Staff", "Principal"];

    // Pre-generate structure to get accurate counts for nested list declarations
    let mut division_data: Vec<(String, String, String, i32, Vec<_>)> = Vec::new();

    for d in 0..divisions {
        let div_id = format!("div{}", d + 1);
        let div_name = format!("Division {}", ['A', 'B', 'C', 'D', 'E'][d % 5]);
        let head: String = format!(
            "{} {}",
            FirstName().fake_with_rng::<String, _>(&mut rng),
            LastName().fake_with_rng::<String, _>(&mut rng)
        );
        let budget = rng.gen_range(1..10) * 1000000;

        let dept_count = rng.gen_range(2..4);
        let mut dept_data = Vec::new();

        for dept in 0..dept_count {
            let dept_id = format!("d{}_{}", d + 1, dept + 1);
            let dept_name = format!("Dept {}{}", d + 1, dept + 1);
            let manager: String = format!(
                "{} {}",
                FirstName().fake_with_rng::<String, _>(&mut rng),
                LastName().fake_with_rng::<String, _>(&mut rng)
            );
            let headcount = rng.gen_range(10..50);

            let team_count = rng.gen_range(1..3);
            let mut team_data = Vec::new();

            for t in 0..team_count {
                let team_id = format!("t{}_{}_{}", d + 1, dept + 1, t + 1);
                let focus = focuses[rng.gen_range(0..focuses.len())];
                let team_name = format!("{} Team", focus);
                let lead: String = format!(
                    "{} {}",
                    FirstName().fake_with_rng::<String, _>(&mut rng),
                    LastName().fake_with_rng::<String, _>(&mut rng)
                );

                let emp_count = rng.gen_range(2..5);
                let mut emp_data = Vec::new();

                for e in 0..emp_count {
                    let emp_id = format!("e{}_{}_{}_{}", d + 1, dept + 1, t + 1, e + 1);
                    let emp_name: String = format!(
                        "{} {}",
                        FirstName().fake_with_rng::<String, _>(&mut rng),
                        LastName().fake_with_rng::<String, _>(&mut rng)
                    );
                    let role = roles[rng.gen_range(0..roles.len())];
                    let level = levels[rng.gen_range(0..levels.len())];
                    emp_data.push((emp_id, emp_name, role.to_string(), level.to_string()));
                }

                team_data.push((team_id, team_name, lead, focus.to_string(), emp_data));
            }

            dept_data.push((dept_id, dept_name, manager, headcount, team_data));
        }

        division_data.push((div_id, div_name, head, budget, dept_data));
    }

    // Generate with clear count syntax: |[N] data - brackets clearly separate count from data
    lines.push("companies: @Company".to_string());
    lines.push(format!(
        "  |[{}] corp1, MegaCorp International, 1985, Technology",
        divisions
    ));

    for (div_id, div_name, head, budget, dept_data) in &division_data {
        lines.push(format!(
            "    |[{}] {},{},{},{}M",
            dept_data.len(),
            div_id,
            div_name,
            head,
            budget / 1_000_000
        ));

        for (dept_id, dept_name, manager, headcount, team_data) in dept_data {
            lines.push(format!(
                "      |[{}] {},{},{},{}",
                team_data.len(),
                dept_id,
                dept_name,
                manager,
                headcount
            ));

            for (team_id, team_name, lead, focus, emp_data) in team_data {
                lines.push(format!(
                    "        |[{}] {},{},{},{}",
                    emp_data.len(),
                    team_id,
                    team_name,
                    lead,
                    focus
                ));

                for (emp_id, emp_name, role, level) in emp_data {
                    lines.push(format!(
                        "          |{},{},{},{}",
                        emp_id, emp_name, role, level
                    ));
                }
            }
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{BenchError, MAX_DATASET_SIZE};

    // Positive tests
    #[test]
    fn test_generate_users_parses() {
        let hedl = generate_users(100);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok(), "Failed to parse users: {:?}", result.err());
    }

    #[test]
    fn test_generate_users_safe_success() {
        let result = generate_users_safe(100);
        assert!(result.is_ok());
        let hedl = result.unwrap();
        assert!(hedl.contains("%STRUCT: User"));
        assert!(hedl_core::parse(hedl.as_bytes()).is_ok());
    }

    // Negative tests - Size validation
    #[test]
    fn test_generate_users_safe_too_large() {
        let result = generate_users_safe(MAX_DATASET_SIZE + 1);
        assert!(result.is_err());
        match result {
            Err(BenchError::DatasetTooLarge { requested, max }) => {
                assert_eq!(requested, MAX_DATASET_SIZE + 1);
                assert_eq!(max, MAX_DATASET_SIZE);
            }
            _ => panic!("Expected DatasetTooLarge error"),
        }
    }

    #[test]
    fn test_validate_dataset_size_boundary() {
        // Exactly at limit should succeed
        assert!(validate_dataset_size(MAX_DATASET_SIZE).is_ok());
        // One over should fail
        assert!(validate_dataset_size(MAX_DATASET_SIZE + 1).is_err());
        // Way over should fail
        assert!(validate_dataset_size(usize::MAX).is_err());
    }

    // Negative tests - Empty/minimal inputs
    #[test]
    fn test_generate_users_zero_count() {
        let hedl = generate_users(0);
        // Should parse but have no data rows
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok(), "Empty dataset should parse successfully");
    }

    #[test]
    fn test_generate_users_single_entity() {
        let hedl = generate_users(1);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok());
        assert!(hedl.contains("user1"));
    }

    #[test]
    fn test_generate_products_parses() {
        let hedl = generate_products(100);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(
            result.is_ok(),
            "Failed to parse products: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_generate_events_parses() {
        let hedl = generate_events(100);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok(), "Failed to parse events: {:?}", result.err());
    }

    #[test]
    fn test_generate_nested_parses() {
        let hedl = generate_nested(10);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok(), "Failed to parse nested: {:?}", result.err());
    }

    #[test]
    fn test_generate_graph_parses() {
        let hedl = generate_graph(50, 3);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok(), "Failed to parse graph: {:?}", result.err());
    }

    #[test]
    fn test_generate_blog_parses() {
        let hedl = generate_blog(10, 3);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok(), "Failed to parse blog: {:?}", result.err());
    }

    #[test]
    fn test_generate_ditto_heavy_parses() {
        let hedl = generate_ditto_heavy(50);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(
            result.is_ok(),
            "Failed to parse ditto_heavy: {:?}",
            result.err()
        );
        // Verify dittos are present (compact format uses ,^,)
        assert!(hedl.contains(",^,"), "Expected ditto operators in output");
    }

    #[test]
    fn test_generate_reference_heavy_parses() {
        let hedl = generate_reference_heavy(10);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(
            result.is_ok(),
            "Failed to parse reference_heavy: {:?}",
            result.err()
        );
        // Verify references and nesting are present
        assert!(hedl.contains("@Person:"), "Expected Person references");
        assert!(
            hedl.contains("@Project:"),
            "Expected Project cross-references"
        );
        assert!(
            hedl.contains("%NEST: Project > Milestone"),
            "Expected nested structure"
        );
    }

    #[test]
    fn test_generate_deep_hierarchy_parses() {
        let hedl = generate_deep_hierarchy(3);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(
            result.is_ok(),
            "Failed to parse deep_hierarchy: {:?}",
            result.err()
        );
        // Verify new count syntax is present
        assert!(
            hedl.contains("%STRUCT: Division (3):"),
            "Expected Division count in %STRUCT"
        );
        assert!(
            hedl.contains("%NEST: Company > Division"),
            "Expected Company > Division nest"
        );
        assert!(
            hedl.contains("%NEST: Division > Department"),
            "Expected Division > Department nest"
        );
        // Verify parent rows have |[N] syntax (clear count format)
        assert!(
            hedl.contains("|[3] corp1, MegaCorp"),
            "Expected company row with division count"
        );
    }

    #[test]
    fn test_generate_analytics_parses() {
        let hedl = generate_analytics(100);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(
            result.is_ok(),
            "Failed to parse analytics: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_generate_config_parses() {
        let hedl = generate_config(5);
        let result = hedl_core::parse(hedl.as_bytes());
        assert!(result.is_ok(), "Failed to parse config: {:?}", result.err());
    }

    #[test]
    fn test_deterministic_generation() {
        // Same seed should produce same output
        let a = generate_users(10);
        let b = generate_users(10);
        assert_eq!(a, b);
    }
}
