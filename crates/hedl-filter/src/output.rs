// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Convert structured command output to HEDL format for maximum token savings.
//!
//! This is HEDL Filter's killer feature - instead of just truncating text,
//! we parse structured command output and re-encode it in HEDL format,
//! achieving 50-80% additional token savings beyond simple filtering.

// HEDL output generation - constructs HEDL text directly

/// Structured data that can be converted to HEDL.
pub struct StructuredData {
    /// Type name for the data
    pub type_name: String,
    /// Field names (schema)
    pub fields: Vec<String>,
    /// Rows of data
    pub rows: Vec<Vec<String>>,
    /// Metadata key-value pairs
    pub metadata: Vec<(String, String)>,
}

impl StructuredData {
    pub fn new(type_name: &str, fields: Vec<String>) -> Self {
        Self {
            type_name: type_name.to_string(),
            fields,
            rows: Vec::new(),
            metadata: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.push((key.to_string(), value.to_string()));
    }
}

/// Convert structured data to HEDL format.
pub fn to_hedl(data: &StructuredData) -> String {
    let mut output = String::new();

    // Header
    output.push_str("%V:2.0\n");
    output.push_str("%NULL:~\n");
    output.push_str("%QUOTE:\"\n");

    // Type definition
    let fields_str = data.fields.join(", ");
    output.push_str(&format!("%S:{}:[{}]\n", data.type_name, fields_str));

    // Metadata as entries
    for (key, value) in &data.metadata {
        output.push_str(&format!("{}: {}\n", key, value));
    }

    output.push_str("---\n");

    // Data section
    output.push_str(&format!("{}: @{}\n", data.type_name.to_lowercase(), data.type_name));
    for row in &data.rows {
        let values = row.join(",");
        output.push_str(&format!(" |{}\n", values));
    }

    output
}

/// Convert git status to HEDL format.
pub fn git_status_to_hedl(branch: &str, staged: &[String], modified: &[String],
    deleted: &[String], untracked: &[String]) -> String {
    let mut data = StructuredData::new("GitStatus", vec![
        "category".to_string(), "file".to_string()
    ]);
    data.add_metadata("branch", branch);

    for f in staged {
        data.add_row(vec!["staged".to_string(), f.clone()]);
    }
    for f in modified {
        data.add_row(vec!["modified".to_string(), f.clone()]);
    }
    for f in deleted {
        data.add_row(vec!["deleted".to_string(), f.clone()]);
    }
    for f in untracked {
        data.add_row(vec!["untracked".to_string(), f.clone()]);
    }

    to_hedl(&data)
}

/// Convert docker ps to HEDL format.
pub fn docker_ps_to_hedl(containers: &[(String, String, String, String)]) -> String {
    let mut data = StructuredData::new("Container", vec![
        "id".to_string(), "image".to_string(), "status".to_string(), "name".to_string()
    ]);

    for (id, image, status, name) in containers {
        data.add_row(vec![id.clone(), image.clone(), status.clone(), name.clone()]);
    }

    to_hedl(&data)
}

/// Convert kubectl pods to HEDL format.
pub fn kubectl_pods_to_hedl(pods: &[(String, String, String, String)]) -> String {
    let mut data = StructuredData::new("Pod", vec![
        "name".to_string(), "ready".to_string(), "status".to_string(), "restarts".to_string()
    ]);

    for (name, ready, status, restarts) in pods {
        data.add_row(vec![name.clone(), ready.clone(), status.clone(), restarts.clone()]);
    }

    to_hedl(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_to_hedl() {
        let hedl = git_status_to_hedl("main",
            &["Cargo.toml".to_string()],
            &["src/main.rs".to_string()],
            &[],
            &["notes.md".to_string()]);

        assert!(hedl.contains("%V:2.0"));
        assert!(hedl.contains("%S:GitStatus:"));
        assert!(hedl.contains("branch: main"));
        assert!(hedl.contains("staged,Cargo.toml"));
        assert!(hedl.contains("modified,src/main.rs"));
        assert!(hedl.contains("untracked,notes.md"));
    }

    #[test]
    fn test_docker_ps_to_hedl() {
        let containers = vec![
            ("abc123".to_string(), "nginx".to_string(), "Up 2h".to_string(), "web".to_string()),
        ];
        let hedl = docker_ps_to_hedl(&containers);
        assert!(hedl.contains("%S:Container:"));
        assert!(hedl.contains("abc123,nginx,Up 2h,web"));
    }
}
