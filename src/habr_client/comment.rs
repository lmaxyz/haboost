use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comments_response() {
        let json = r#"{
            "cacheKey": "0421ba0c-8586-45c5-b925-0fb65568d772",
            "lastCommentTimestamp": 1770763737,
            "threads": ["29430966", "29430978"],
            "commentIds": ["29430966", "29432434"],
            "commentRefs": {
                "29430966": {
                    "id": "29430966",
                    "parentId": null,
                    "level": 0,
                    "timePublished": "2026-01-25T08:09:55+00:00",
                    "timeChanged": null,
                    "isSuspended": false,
                    "status": "published",
                    "score": 15,
                    "votesCount": 15,
                    "message": "<p>Test comment</p>",
                    "editorVersion": 2,
                    "author": {
                        "id": "5510585",
                        "alias": "TestUser",
                        "fullname": "Test User",
                        "avatarUrl": null,
                        "speciality": null
                    },
                    "isAuthor": false,
                    "isPostAuthor": false,
                    "isNew": false,
                    "isFavorite": false,
                    "isCanEdit": false,
                    "timeEditAllowedTill": null,
                    "children": ["29432434"],
                    "vote": null,
                    "votePlus": null,
                    "voteMinus": null,
                    "isPinned": false
                },
                "29432434": {
                    "id": "29432434",
                    "parentId": "29430966",
                    "level": 1,
                    "timePublished": "2026-01-25T09:00:00+00:00",
                    "timeChanged": null,
                    "isSuspended": false,
                    "status": "published",
                    "score": 5,
                    "votesCount": 5,
                    "message": "<p>Reply comment</p>",
                    "editorVersion": 2,
                    "author": {
                        "id": "1234567",
                        "alias": "ReplyUser",
                        "fullname": "Reply User",
                        "avatarUrl": "https://habr.com/avatar.png",
                        "speciality": null
                    },
                    "isAuthor": false,
                    "isPostAuthor": false,
                    "isNew": false,
                    "isFavorite": false,
                    "isCanEdit": false,
                    "timeEditAllowedTill": null,
                    "children": [],
                    "vote": null,
                    "votePlus": null,
                    "voteMinus": null,
                    "isPinned": false
                }
            },
            "pinnedCommentIds": []
        }"#;

        let response: CommentsResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.threads.len(), 2);
        assert_eq!(response.comment_refs.len(), 2);

        let root_comment = response.comment_refs.get("29430966").unwrap();
        assert_eq!(root_comment.id, "29430966");
        assert_eq!(root_comment.author.alias, "TestUser");
        assert_eq!(root_comment.score, 15);
        assert_eq!(root_comment.level, 0);
        assert!(root_comment.parent_id.is_none());

        let reply_comment = response.comment_refs.get("29432434").unwrap();
        assert_eq!(reply_comment.author.alias, "ReplyUser");
        assert_eq!(reply_comment.level, 1);
        assert_eq!(reply_comment.parent_id.as_deref(), Some("29430966"));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    pub id: String,
    #[serde(rename(deserialize = "parentId"))]
    pub parent_id: Option<String>,
    pub level: usize,
    #[serde(rename(deserialize = "timePublished"))]
    pub published_at: String,
    pub message: String,
    pub score: isize,
    pub author: CommentAuthor,
    #[serde(rename(deserialize = "children"))]
    pub children_ids: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub children: Vec<Comment>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommentAuthor {
    pub alias: String,
    #[serde(rename(deserialize = "avatarUrl"))]
    pub avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentsResponse {
    #[serde(rename(deserialize = "commentRefs"))]
    pub comment_refs: HashMap<String, Comment>,
    #[serde(rename(deserialize = "threads"))]
    pub threads: Vec<String>,
}
