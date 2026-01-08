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

//! String manipulation utilities for HEDL format conversion.

/// Singularize common English plurals and convert to PascalCase.
///
/// This function is useful when converting from formats like JSON, XML, or YAML
/// where collection keys are often pluralized (e.g., "users", "posts") but
/// HEDL struct types should be singular PascalCase (e.g., "User", "Post").
///
/// # Examples
///
/// ```
/// use hedl_core::lex::singularize_and_capitalize;
///
/// // Simple plurals
/// assert_eq!(singularize_and_capitalize("users"), "User");
/// assert_eq!(singularize_and_capitalize("posts"), "Post");
///
/// // -ies plurals
/// assert_eq!(singularize_and_capitalize("categories"), "Category");
///
/// // -es plurals
/// assert_eq!(singularize_and_capitalize("boxes"), "Box");
/// assert_eq!(singularize_and_capitalize("classes"), "Class");
///
/// // Snake case to PascalCase
/// assert_eq!(singularize_and_capitalize("user_posts"), "UserPost");
/// assert_eq!(singularize_and_capitalize("alias_contexts"), "AliasContext");
///
/// // Already singular
/// assert_eq!(singularize_and_capitalize("user"), "User");
/// assert_eq!(singularize_and_capitalize("data"), "Data");
/// ```
pub fn singularize_and_capitalize(s: &str) -> String {
    let singular = if s.ends_with("ies") && s.len() > 3 {
        // categories -> category
        format!("{}y", &s[..s.len() - 3])
    } else if s.ends_with("es") && s.len() > 2 {
        // boxes -> box, but also handles "classes" -> "class"
        let base = &s[..s.len() - 2];
        if base.ends_with("ss")
            || base.ends_with("sh")
            || base.ends_with("ch")
            || base.ends_with('x')
        {
            base.to_string()
        } else {
            // Just remove 's' for regular cases like "types" -> "type"
            s[..s.len() - 1].to_string()
        }
    } else if s.ends_with('s') && s.len() > 1 {
        // users -> user
        s[..s.len() - 1].to_string()
    } else {
        s.to_string()
    };

    // Convert snake_case to PascalCase
    singular
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_plurals() {
        assert_eq!(singularize_and_capitalize("users"), "User");
        assert_eq!(singularize_and_capitalize("posts"), "Post");
        assert_eq!(singularize_and_capitalize("items"), "Item");
    }

    #[test]
    fn test_ies_plurals() {
        assert_eq!(singularize_and_capitalize("categories"), "Category");
        assert_eq!(singularize_and_capitalize("entries"), "Entry");
        assert_eq!(singularize_and_capitalize("queries"), "Query");
    }

    #[test]
    fn test_es_plurals() {
        assert_eq!(singularize_and_capitalize("boxes"), "Box");
        assert_eq!(singularize_and_capitalize("classes"), "Class");
        assert_eq!(singularize_and_capitalize("matches"), "Match");
        assert_eq!(singularize_and_capitalize("bushes"), "Bush");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(singularize_and_capitalize("user_posts"), "UserPost");
        assert_eq!(singularize_and_capitalize("alias_contexts"), "AliasContext");
        assert_eq!(singularize_and_capitalize("my_items"), "MyItem");
    }

    #[test]
    fn test_already_singular() {
        assert_eq!(singularize_and_capitalize("user"), "User");
        assert_eq!(singularize_and_capitalize("data"), "Data");
        assert_eq!(singularize_and_capitalize("info"), "Info");
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(singularize_and_capitalize("s"), "S");
        assert_eq!(singularize_and_capitalize(""), "");
        assert_eq!(singularize_and_capitalize("_"), "");
        // "__users__" singularizes the whole string first (no 's' suffix due to trailing '_')
        // then splits by '_' which filters empty parts: ["users"] -> "Users"
        assert_eq!(singularize_and_capitalize("__users__"), "Users");
    }

    // ==================== Additional singular tests ====================

    #[test]
    fn test_already_pascal_case() {
        // Already PascalCase inputs
        assert_eq!(singularize_and_capitalize("User"), "User");
        assert_eq!(singularize_and_capitalize("UserPost"), "UserPost");
    }

    #[test]
    fn test_single_letter() {
        assert_eq!(singularize_and_capitalize("a"), "A");
        assert_eq!(singularize_and_capitalize("x"), "X");
    }

    #[test]
    fn test_two_letters() {
        assert_eq!(singularize_and_capitalize("us"), "U");
        assert_eq!(singularize_and_capitalize("ax"), "Ax");
    }

    #[test]
    fn test_words_ending_in_ss() {
        // Words ending in "ss" still get singularized (remove last 's')
        // This is a simple heuristic, not perfect English grammar
        assert_eq!(singularize_and_capitalize("class"), "Clas");
        assert_eq!(singularize_and_capitalize("pass"), "Pas");
        assert_eq!(singularize_and_capitalize("boss"), "Bos");
        // For "classes" though, the -es rule handles it correctly
        assert_eq!(singularize_and_capitalize("classes"), "Class");
    }

    #[test]
    fn test_words_ending_in_us() {
        // Words ending in "us" - remove 's'
        assert_eq!(singularize_and_capitalize("status"), "Statu");
        assert_eq!(singularize_and_capitalize("bonus"), "Bonu");
    }

    #[test]
    fn test_words_ending_in_is() {
        assert_eq!(singularize_and_capitalize("basis"), "Basi");
        assert_eq!(singularize_and_capitalize("analysis"), "Analysi");
    }

    #[test]
    fn test_foxes_type_words() {
        // Words like "foxes" -> "fox"
        assert_eq!(singularize_and_capitalize("foxes"), "Fox");
        assert_eq!(singularize_and_capitalize("taxes"), "Tax");
        assert_eq!(singularize_and_capitalize("indexes"), "Index");
    }

    #[test]
    fn test_wishes_type_words() {
        // Words like "wishes" -> "wish"
        assert_eq!(singularize_and_capitalize("wishes"), "Wish");
        assert_eq!(singularize_and_capitalize("dishes"), "Dish");
    }

    #[test]
    fn test_watches_type_words() {
        // Words like "watches" -> "watch"
        assert_eq!(singularize_and_capitalize("watches"), "Watch");
        assert_eq!(singularize_and_capitalize("catches"), "Catch");
    }

    #[test]
    fn test_types_word() {
        // "types" -> "type" (es -> e, not es -> empty)
        assert_eq!(singularize_and_capitalize("types"), "Type");
        assert_eq!(singularize_and_capitalize("files"), "File");
        assert_eq!(singularize_and_capitalize("names"), "Name");
    }

    // ==================== Snake case conversion tests ====================

    #[test]
    fn test_deep_snake_case() {
        assert_eq!(singularize_and_capitalize("a_b_c_d_e"), "ABCDE");
    }

    #[test]
    fn test_snake_case_with_numbers() {
        assert_eq!(singularize_and_capitalize("user_v2"), "UserV2");
        assert_eq!(singularize_and_capitalize("api_v1_users"), "ApiV1User");
    }

    #[test]
    fn test_leading_underscore_snake() {
        assert_eq!(singularize_and_capitalize("_items"), "Item");
    }

    #[test]
    fn test_trailing_underscore_snake() {
        // "items_" doesn't end in 's', so singularization doesn't happen
        // Only the snake_case -> PascalCase conversion occurs
        assert_eq!(singularize_and_capitalize("items_"), "Items");
    }

    #[test]
    fn test_multiple_underscores() {
        assert_eq!(singularize_and_capitalize("some__items"), "SomeItem");
        assert_eq!(singularize_and_capitalize("many___things"), "ManyThing");
    }

    #[test]
    fn test_mixed_case_snake() {
        // Mixed case in snake_case parts - only first char is uppercased
        // The function preserves case of remaining chars
        assert_eq!(singularize_and_capitalize("user_IDs"), "UserID");
        // PARSERS ends in uppercase 'S', not lowercase 's'
        // So singularization doesn't trigger (case-sensitive)
        assert_eq!(singularize_and_capitalize("xml_PARSERS"), "XmlPARSERS");
    }

    // ==================== Ies plurals extended ====================

    #[test]
    fn test_ies_short_word() {
        // "ies" at the end of short words
        assert_eq!(singularize_and_capitalize("flies"), "Fly");
        assert_eq!(singularize_and_capitalize("ties"), "Ty");
    }

    #[test]
    fn test_ies_exact_length() {
        // Word exactly "ies" (len = 3) should NOT be singularized by ies rule
        assert_eq!(singularize_and_capitalize("ies"), "Ie");
    }

    // ==================== Es plurals extended ====================

    #[test]
    fn test_es_on_short_words() {
        // "es" on short words
        assert_eq!(singularize_and_capitalize("es"), "E");
    }

    #[test]
    fn test_es_various() {
        assert_eq!(singularize_and_capitalize("heroes"), "Heroe");
        assert_eq!(singularize_and_capitalize("potatoes"), "Potatoe");
        assert_eq!(singularize_and_capitalize("tomatoes"), "Tomatoe");
    }

    // ==================== Complex combinations ====================

    #[test]
    fn test_snake_plus_ies() {
        assert_eq!(
            singularize_and_capitalize("user_categories"),
            "UserCategory"
        );
        assert_eq!(singularize_and_capitalize("post_entries"), "PostEntry");
    }

    #[test]
    fn test_snake_plus_es() {
        assert_eq!(singularize_and_capitalize("data_boxes"), "DataBox");
        assert_eq!(singularize_and_capitalize("file_matches"), "FileMatch");
    }

    #[test]
    fn test_very_long_snake_case() {
        assert_eq!(
            singularize_and_capitalize("this_is_a_very_long_identifier_with_many_parts"),
            "ThisIsAVeryLongIdentifierWithManyPart"
        );
    }

    // ==================== Unicode edge cases ====================

    #[test]
    fn test_unicode_preserved() {
        // Unicode chars should pass through
        assert_eq!(singularize_and_capitalize("日本語s"), "日本語");
        assert_eq!(singularize_and_capitalize("émojis"), "Émoji");
    }

    #[test]
    fn test_all_caps() {
        // Uppercase 'S' doesn't trigger singularization (case-sensitive)
        assert_eq!(singularize_and_capitalize("USERS"), "USERS");
        assert_eq!(singularize_and_capitalize("ITEMS"), "ITEMS");
        // Only lowercase 's' at end triggers singularization
        assert_eq!(singularize_and_capitalize("users"), "User");
    }

    #[test]
    fn test_mixed_caps() {
        assert_eq!(singularize_and_capitalize("myItems"), "MyItem");
        assert_eq!(singularize_and_capitalize("SomeThings"), "SomeThing");
    }
}
