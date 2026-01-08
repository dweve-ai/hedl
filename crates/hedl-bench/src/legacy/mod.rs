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

//! Legacy modules preserved for backward compatibility.
//!
//! These modules are deprecated and will be removed in a future version.
//! Please migrate to the new modular structure in generators/ and helpers/.

#[deprecated(since = "0.3.0", note = "Use generators and helpers modules instead")]
pub mod accuracy;

#[deprecated(since = "0.3.0", note = "Use generators and helpers modules instead")]
pub mod normalize;

#[deprecated(since = "0.3.0", note = "Use generators and helpers modules instead")]
pub mod questions;
