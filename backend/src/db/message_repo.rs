use mongodb::{
    bson::{doc, oid::ObjectId, to_bson, Document},
    options::{FindOneAndUpdateOptions, FindOptions, IndexOptions, ReturnDocument},
    Collection, IndexModel,
};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub emoji: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageDb {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<Reaction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_at: Option<String>,
}

pub struct MessageRepo {
    collection: Option<Collection<MessageDb>>,
}

impl MessageRepo {
    fn escape_mongo_regex(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '\\' | '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
                    out.push('\\');
                    out.push(ch);
                }
                _ => out.push(ch),
            }
        }
        out
    }

    pub fn new(collection: Collection<MessageDb>) -> Self {
        let repo = Self { collection: Some(collection) };
        if let Some(ref coll) = repo.collection {
            tokio::spawn(Self::create_indexes(coll.clone()));
        }
        repo
    }

    /// Create a placeholder repo when MongoDB is not configured
    pub fn new_placeholder() -> Self {
        Self { collection: None }
    }

    // ---------------------------------------------------------
    // 📌 Création des indexes MongoDB (automatique au démarrage)
    // ---------------------------------------------------------
    async fn create_indexes(collection: Collection<MessageDb>) {
        // Index: channel_id ASC
        let idx_channel = IndexModel::builder()
            .keys(doc! { "channel_id": 1 })
            .options(
                IndexOptions::builder()
                    .name("channel_id_1".to_string())
                    .build(),
            )
            .build();

        // Index: created_at DESC
        let idx_created = IndexModel::builder()
            .keys(doc! { "created_at": -1 })
            .options(
                IndexOptions::builder()
                    .name("created_at_-1".to_string())
                    .build(),
            )
            .build();

        // Index: pinned_at DESC (for fast pinned retrieval)
        let idx_pinned_at = IndexModel::builder()
            .keys(doc! { "pinned_at": -1 })
            .options(
                IndexOptions::builder()
                    .name("pinned_at_-1".to_string())
                    .build(),
            )
            .build();

        let _ = collection.create_index(idx_channel, None).await;
        let _ = collection.create_index(idx_created, None).await;
        let _ = collection.create_index(idx_pinned_at, None).await;

        // Text index for content to support full-text search
        let idx_text = IndexModel::builder()
            .keys(doc! { "content": "text" })
            .options(
                IndexOptions::builder()
                    .name("content_text".to_string())
                    .build(),
            )
            .build();
        let _ = collection.create_index(idx_text, None).await;

        println!("✅ MongoDB indexes created");
    }

    // ---------------------------------------------------------
    // 📥 INSERT MESSAGE
    // ---------------------------------------------------------
    pub async fn insert(&self, msg: MessageDb) -> mongodb::error::Result<ObjectId> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| mongodb::error::Error::custom("MongoDB not configured"))?;
        let result = collection.insert_one(msg, None).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_by_id(&self, message_id: &ObjectId) -> Option<MessageDb> {
        let collection = self.collection.as_ref()?;
        match collection.find_one(doc! { "_id": message_id }, None).await {
            Ok(Some(msg)) => Some(msg),
            _ => None,
        }
    }

    pub async fn add_reaction(
        &self,
        message_id: &ObjectId,
        emoji: &str,
        user_id: &str,
        username: Option<&str>,
    ) -> mongodb::error::Result<(Option<String>, bool)> {
        let collection = self.collection.as_ref().ok_or_else(|| {
            mongodb::error::Error::custom("MongoDB not configured")
        })?;

        let existing = collection
            .find_one(doc! { "_id": message_id }, None)
            .await?;

        let msg = match existing {
            Some(m) => m,
            None => return Ok((None, false)),
        };

        let mut reactions = msg.reactions.unwrap_or_default();
        let idx = reactions.iter().position(|r| r.emoji == emoji && r.user_id == user_id);

        let was_added = if let Some(pos) = idx {
            reactions.remove(pos);
            false
        } else {
            reactions.push(Reaction {
                emoji: emoji.to_string(),
                user_id: user_id.to_string(),
                username: username.map(|u| u.to_string()),
                created_at: chrono::Utc::now().to_rfc3339(),
            });
            true
        };

        collection
            .update_one(
                doc! { "_id": message_id },
                doc! { "$set": { "reactions": to_bson(&reactions)? } },
                None,
            )
            .await?;

        Ok((Some(msg.channel_id), was_added))
    }

    pub async fn remove_reaction(
        &self,
        message_id: &ObjectId,
        emoji: &str,
        user_id: &str,
    ) -> mongodb::error::Result<Option<String>> {
        let collection = self.collection.as_ref().ok_or_else(|| {
            mongodb::error::Error::custom("MongoDB not configured")
        })?;

        let existing = collection
            .find_one(doc! { "_id": message_id }, None)
            .await?;

        let msg = match existing {
            Some(m) => m,
            None => return Ok(None),
        };

        let reactions = msg.reactions.unwrap_or_default();
        let filtered: Vec<Reaction> = reactions
            .into_iter()
            .filter(|r| !(r.emoji == emoji && r.user_id == user_id))
            .collect();

        collection
            .update_one(
                doc! { "_id": message_id },
                doc! { "$set": { "reactions": to_bson(&filtered)? } },
                None,
            )
            .await?;

        Ok(Some(msg.channel_id))
    }

    pub async fn delete(&self, message_id: &ObjectId) -> mongodb::error::Result<bool> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| mongodb::error::Error::custom("MongoDB not configured"))?;
        let result = collection.delete_one(doc! { "_id": message_id }, None).await?;
        Ok(result.deleted_count > 0)
    }

    // ---------------------------------------------------------
    // 📜 GET HISTORY (trié par created_at DESC) avec pagination
    // params: page (1-based), per_page
    // ---------------------------------------------------------
    pub async fn find_by_channel(&self, channel_id: &str, page: u64, per_page: u64) -> Vec<MessageDb> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return vec![],
        };

        let skip = if page == 0 { 0 } else { (page - 1) * per_page };
        let limit = per_page;

        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(Some(skip))
            .limit(Some(limit as i64))
            .build();

        let mut cursor = match collection
            .find(doc! { "channel_id": channel_id }, options)
            .await
        {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut messages = Vec::new();
        while let Some(msg) = cursor.try_next().await.unwrap_or(None) {
            messages.push(msg);
        }

        // On renvoie l'historique dans l'ordre chronologique
        messages.reverse();
        messages
    }

    // ---------------------------------------------------------
    // 📜 GET HISTORY BEFORE cursor (cursor-based pagination)
    // Returns per_page messages with _id < before_id, chronological order
    // ---------------------------------------------------------
    pub async fn find_by_channel_before(
        &self,
        channel_id: &str,
        before_id: &ObjectId,
        per_page: u64,
    ) -> Vec<MessageDb> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return vec![],
        };

        let filter = doc! {
            "channel_id": channel_id,
            "_id": { "$lt": before_id },
        };
        let options = FindOptions::builder()
            .sort(doc! { "_id": -1 })
            .limit(Some(per_page as i64))
            .build();

        let mut cursor = match collection.find(filter, options).await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut messages = Vec::new();
        while let Some(msg) = cursor.try_next().await.unwrap_or(None) {
            messages.push(msg);
        }

        messages.reverse();
        messages
    }

    // ---------------------------------------------------------
    // � SEARCH IN CHANNEL (case-insensitive, simple regex)
    // ---------------------------------------------------------
    pub async fn search_in_channel(&self, channel_id: &str, query: &str, page: u64, per_page: u64) -> Vec<MessageDb> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return vec![],
        };

        let skip = if page == 0 { 0 } else { (page - 1) * per_page };
        let limit = per_page;

        // Prefer text search if possible (faster & better relevance). Fall back to regex if needed.
        let filter = doc! {
            "channel_id": channel_id,
            "$text": { "$search": query }
        };

        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(Some(skip))
            .limit(Some(limit as i64))
            .build();

        let mut cursor = match collection.find(filter, options).await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut messages = Vec::new();
        while let Some(msg) = cursor.try_next().await.unwrap_or(None) {
            messages.push(msg);
        }

        // Return chronological order (oldest first)
        messages.reverse();
        messages
    }

    // ---------------------------------------------------------
    // �📬 GET DM CONVERSATIONS for a user
    // Returns a list of { conversation_id, last_message, last_message_at, peer_id }
    // ---------------------------------------------------------
    pub async fn search_by_channel_content(
        &self,
        channel_id: &str,
        query: &str,
        page: u64,
        per_page: u64,
    ) -> Vec<MessageDb> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return vec![],
        };

        let skip = if page == 0 { 0 } else { (page - 1) * per_page };
        let limit = per_page;
        let escaped_query = Self::escape_mongo_regex(query);

        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip(Some(skip))
            .limit(Some(limit as i64))
            .build();

        let filter = doc! {
            "channel_id": channel_id,
            "content": {
                "$regex": escaped_query,
                "$options": "i",
            }
        };

        let mut cursor = match collection.find(filter, options).await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut messages = Vec::new();
        while let Some(msg) = cursor.try_next().await.unwrap_or(None) {
            messages.push(msg);
        }

        messages.reverse();
        messages
    }

    // ---------------------------------------------------------
    // ✏️ EDIT MESSAGE (author-only)
    // ---------------------------------------------------------
    pub async fn update_content(
        &self,
        message_id: ObjectId,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> mongodb::error::Result<Option<MessageDb>> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return Ok(None),
        };

        let filter = doc! {
            "_id": message_id,
            "channel_id": channel_id,
            "author_id": author_id,
        };
        let update = doc! {
            "$set": {
                "content": content,
            }
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(Some(ReturnDocument::After))
            .build();

        collection.find_one_and_update(filter, update, options).await
    }

    // ---------------------------------------------------------
    // 📌 PIN / UNPIN MESSAGE
    // ---------------------------------------------------------
    pub async fn pin_message(
        &self,
        message_id: ObjectId,
        channel_id: &str,
        pinned_by: &str,
        pinned_at: &str,
    ) -> mongodb::error::Result<Option<MessageDb>> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return Ok(None),
        };

        let filter = doc! {
            "_id": message_id,
            "channel_id": channel_id,
        };
        let update = doc! {
            "$set": {
                "pinned_by": pinned_by,
                "pinned_at": pinned_at,
            }
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(Some(ReturnDocument::After))
            .build();

        collection.find_one_and_update(filter, update, options).await
    }

    pub async fn unpin_message(
        &self,
        message_id: ObjectId,
        channel_id: &str,
    ) -> mongodb::error::Result<Option<MessageDb>> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return Ok(None),
        };

        let filter = doc! {
            "_id": message_id,
            "channel_id": channel_id,
        };
        let update = doc! {
            "$unset": {
                "pinned_by": "",
                "pinned_at": "",
            }
        };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(Some(ReturnDocument::After))
            .build();

        collection.find_one_and_update(filter, update, options).await
    }

    pub async fn find_pinned_by_channel(
        &self,
        channel_id: &str,
        page: u64,
        per_page: u64,
    ) -> Vec<MessageDb> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return vec![],
        };

        let skip = if page == 0 { 0 } else { (page - 1) * per_page };
        let limit = per_page;

        let options = FindOptions::builder()
            .sort(doc! { "pinned_at": -1 })
            .skip(Some(skip))
            .limit(Some(limit as i64))
            .build();

        let filter = doc! {
            "channel_id": channel_id,
            "pinned_at": { "$exists": true },
        };

        let mut cursor = match collection.find(filter, options).await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut messages = Vec::new();
        while let Some(msg) = cursor.try_next().await.unwrap_or(None) {
            messages.push(msg);
        }

        messages.reverse();
        messages
    }

    pub async fn get_dm_conversations(&self, user_id: &str) -> Vec<Document> {
        let collection = match self.collection.as_ref() {
            Some(c) => c,
            None => return vec![],
        };

        let escaped_user_id = Self::escape_mongo_regex(user_id);
        let conversation_pattern = format!(
            "^dm:({uid}:[^:]+|[^:]+:{uid})$",
            uid = escaped_user_id
        );

        let pipeline = vec![
            doc! {
                "$match": {
                    "channel_id": {
                        "$regex": conversation_pattern,
                    }
                }
            },
            doc! { "$sort": { "created_at": -1 } },
            doc! {
                "$group": {
                    "_id": "$channel_id",
                    "last_message": { "$first": "$content" },
                    "last_message_at": { "$first": "$created_at" },
                }
            },
            doc! { "$sort": { "last_message_at": -1 } },
        ];

        let mut cursor = match collection.aggregate(pipeline, None).await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut conversations = Vec::new();
        while let Some(doc) = cursor.try_next().await.unwrap_or(None) {
            conversations.push(doc);
        }

        conversations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn insert_on_placeholder_repo_returns_error() {
        let repo = MessageRepo::new_placeholder();
        let msg = MessageDb {
            id: None,
            channel_id: "chan".into(),
            author_id: "user".into(),
            content: "hello".into(),
            created_at: "now".into(),
            reply_to: None,
            attachment_url: None,
            reactions: None,
            pinned_by: None,
            pinned_at: None,
        };

        let res = repo.insert(msg).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn find_by_channel_on_placeholder_returns_empty() {
        let repo = MessageRepo::new_placeholder();
        let res = repo.find_by_channel("chan", 1, 20).await;
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn update_content_on_placeholder_returns_none() {
        let repo = MessageRepo::new_placeholder();
        let res = repo
            .update_content(ObjectId::new(), "chan", "user", "edited")
            .await
            .expect("placeholder edit should not error");
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn search_by_channel_content_on_placeholder_returns_empty() {
        let repo = MessageRepo::new_placeholder();
        let res = repo
            .search_by_channel_content("chan", "hello", 1, 20)
            .await;
        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn search_in_channel_on_placeholder_returns_empty() {
        let repo = MessageRepo::new_placeholder();
        let res = repo.search_in_channel("chan", "hello", 1, 20).await;
        assert!(res.is_empty());
    }

    #[test]
    fn escape_mongo_regex_escapes_special_chars() {
        let raw = "a+b*(c)?[d]{e}|f.g\\h^i$j";
        let escaped = MessageRepo::escape_mongo_regex(raw);
        assert_eq!(escaped, "a\\+b\\*\\(c\\)\\?\\[d\\]\\{e\\}\\|f\\.g\\\\h\\^i\\$j");
    }

    #[tokio::test]
    async fn pin_unpin_and_list_pinned_on_placeholder_are_noops() {
        let repo = MessageRepo::new_placeholder();
        let id = ObjectId::new();

        let pinned = repo
            .pin_message(id, "chan", "user", "2026-03-20T00:00:00Z")
            .await
            .expect("pin on placeholder should not error");
        assert!(pinned.is_none());

        let unpinned = repo
            .unpin_message(id, "chan")
            .await
            .expect("unpin on placeholder should not error");
        assert!(unpinned.is_none());

        let listed = repo.find_pinned_by_channel("chan", 1, 20).await;
        assert!(listed.is_empty());
    }
}