//! Async tests for inline child list format `@Type#N:|child1|child2|...`

#[cfg(feature = "async")]
mod async_tests {
    use hedl_stream::{AsyncStreamingParser, NodeEvent};
    use std::io::Cursor;

    #[tokio::test]
    async fn test_async_inline_children_basic() {
        let input = r#"%VERSION: 2.0
%STRUCT: Task: [id,title]
%STRUCT: Comment: [id,author,text]
%NEST: Task>Comment
---
tasks:@Task
 |task-001,Review rate limiting
  @Comment#2:|cmt-001,@emp-007,Performance metrics look promising|cmt-002,@emp-011,Found an issue
"#;

        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let mut nodes = Vec::new();
        while let Some(event) = parser.next_event().await.unwrap() {
            if let NodeEvent::Node(node) = event {
                nodes.push(node);
            }
        }

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

    #[tokio::test]
    async fn test_async_inline_children_max_allowed() {
        // Async parser enforces max of 5 inline children
        // Test that 5 children works correctly
        let input = r#"%VERSION: 2.0
%STRUCT: Task: [id,title]
%STRUCT: Comment: [id,text]
%NEST: Task>Comment
---
tasks:@Task
 |task-001,Test task
  @Comment#5:|c1,t1|c2,t2|c3,t3|c4,t4|c5,t5
"#;

        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let mut nodes = Vec::new();
        while let Some(event) = parser.next_event().await.unwrap() {
            if let NodeEvent::Node(node) = event {
                nodes.push(node);
            }
        }

        let comments: Vec<_> = nodes.iter().filter(|n| n.type_name == "Comment").collect();

        assert_eq!(nodes.len(), 6, "Expected 1 task + 5 comments");
        assert_eq!(comments.len(), 5, "Expected 5 comment nodes");

        // Verify all comment IDs are correct
        for (i, comment) in comments.iter().enumerate() {
            assert_eq!(comment.id, format!("c{}", i + 1));
            assert_eq!(comment.parent_id, Some("task-001".to_string()));
        }
    }
}
