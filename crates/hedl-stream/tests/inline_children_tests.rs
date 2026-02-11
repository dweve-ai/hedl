//! Tests for inline child list format `@Type#N:|child1|child2|...`

use hedl_stream::StreamingParser;
use std::io::Cursor;

#[test]
fn test_inline_children_basic() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,author,text]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,Review rate limiting
  @Comment#2:|cmt-001,@emp-007,Performance metrics look promising|cmt-002,@emp-011,Found an issue
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Count different event types
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    let comments: Vec<_> = nodes.iter().filter(|n| n.type_name == "Comment").collect();

    assert_eq!(nodes.len(), 3, "Expected 1 task + 2 comments");
    assert_eq!(comments.len(), 2, "Expected 2 comment nodes");

    // Verify parent-child relationships
    assert!(comments[0].is_nested(), "Comments should be nested");
    assert_eq!(comments[0].parent_id, Some("task-001".to_string()));
    assert_eq!(comments[0].parent_type, Some("Task".to_string()));

    // Verify comment IDs
    assert_eq!(comments[0].id, "cmt-001");
    assert_eq!(comments[1].id, "cmt-002");
}

#[test]
fn test_inline_children_multiple_tasks() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,author,text]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,First task
  @Comment#2:|cmt-001,@emp-001,Comment 1|cmt-002,@emp-002,Comment 2
 |task-002,Second task
  @Comment#3:|cmt-003,@emp-003,Comment 3|cmt-004,@emp-004,Comment 4|cmt-005,@emp-005,Comment 5
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    let comments: Vec<_> = nodes.iter().filter(|n| n.type_name == "Comment").collect();

    assert_eq!(nodes.len(), 7, "Expected 2 tasks + 5 comments");
    assert_eq!(comments.len(), 5, "Expected 5 comment nodes");

    // Verify first task's comments
    let task1_comments: Vec<_> = comments
        .iter()
        .filter(|c| c.parent_id == Some("task-001".to_string()))
        .collect();
    assert_eq!(task1_comments.len(), 2);

    // Verify second task's comments
    let task2_comments: Vec<_> = comments
        .iter()
        .filter(|c| c.parent_id == Some("task-002".to_string()))
        .collect();
    assert_eq!(task2_comments.len(), 3);
}

#[test]
fn test_inline_children_explicit_values() {
    // In v2.0, ditto is not allowed - all values must be explicit
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,author,text]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,Test task
  @Comment#3:|cmt-001,@emp-001,First comment|cmt-002,@emp-001,Second comment|cmt-003,@emp-001,Third comment
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    let comments: Vec<_> = nodes.iter().filter(|n| n.type_name == "Comment").collect();

    assert_eq!(comments.len(), 3, "Expected 3 comment nodes");

    // Verify all have the same author (explicit values, not ditto)
    use hedl_core::Value;
    let expected_ref = Value::Reference(hedl_core::Reference {
        type_name: None,
        id: "emp-001".to_string().into(),
    });

    assert_eq!(comments[0].get_field(1), Some(&expected_ref));
    assert_eq!(comments[1].get_field(1), Some(&expected_ref));
    assert_eq!(comments[2].get_field(1), Some(&expected_ref));
}

#[test]
fn test_inline_children_many_items() {
    // Per SPEC, the inline limit is a "style rule (not a hard syntax limit)"
    // So we test that parsing works correctly with many inline children
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,text]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,Test task
  @Comment#6:|c1,t1|c2,t2|c3,t3|c4,t4|c5,t5|c6,t6
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    let comments: Vec<_> = nodes.iter().filter(|n| n.type_name == "Comment").collect();

    assert_eq!(nodes.len(), 7, "Expected 1 task + 6 comments");
    assert_eq!(comments.len(), 6, "Expected 6 comment nodes");

    // Verify all comment IDs are correct
    for (i, comment) in comments.iter().enumerate() {
        assert_eq!(comment.id, format!("c{}", i + 1));
        assert_eq!(comment.parent_id, Some("task-001".to_string()));
    }
}

#[test]
fn test_inline_children_count_mismatch() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,text]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,Test task
  @Comment#3:|c1,t1|c2,t2
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let mut error_found = false;
    for event in parser {
        if let Err(e) = event {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("count mismatch"),
                "Should error on count mismatch: {}",
                err_msg
            );
            error_found = true;
            break;
        }
    }

    assert!(error_found, "Should have errored for count mismatch");
}

#[test]
fn test_inline_children_zero_count() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,text]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,Test task
  @Comment#0:|
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    let comments: Vec<_> = nodes.iter().filter(|n| n.type_name == "Comment").collect();

    assert_eq!(comments.len(), 0, "Expected no comments for #0");
}

#[test]
fn test_inline_children_no_nest_rule() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,text]
---
tasks:@Task
 |task-001,Test task
  @Comment#1:|c1,t1
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let mut error_found = false;
    for event in parser {
        if let Err(e) = event {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("NEST rule"),
                "Should error on missing NEST: {}",
                err_msg
            );
            error_found = true;
            break;
        }
    }

    assert!(error_found, "Should have errored for missing NEST rule");
}

#[test]
fn test_inline_children_wrong_child_type() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,text]
%S:Tag:[id,name]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,Test task
  @Tag#1:|tag1,Important
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let mut error_found = false;
    for event in parser {
        if let Err(e) = event {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("not a declared child"),
                "Should error on wrong child type: {}",
                err_msg
            );
            error_found = true;
            break;
        }
    }

    assert!(error_found, "Should have errored for wrong child type");
}

#[test]
fn test_inline_children_with_comment() {
    let input = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:Task:[id,title]
%S:Comment:[id,text]
%NEST:Task>Comment
---
tasks:@Task
 |task-001,Test task
  @Comment#2:|c1,t1|c2,t2 # This is a comment
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    let comments: Vec<_> = nodes.iter().filter(|n| n.type_name == "Comment").collect();

    assert_eq!(
        comments.len(),
        2,
        "Expected 2 comments (inline comment should be stripped)"
    );
}
