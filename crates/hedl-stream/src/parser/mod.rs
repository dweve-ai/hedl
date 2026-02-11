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

//! Streaming parser implementation

mod config;
mod context;
mod core;
mod directives;
pub(crate) mod helpers;
mod list_parsing;
mod value_inference;

// Re-export public API
pub use config::StreamingParserConfig;
pub use core::StreamingParser;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_basic_parsing() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
        assert!(!events.is_empty());
    }
}
