use crate::db::message_repo::{MessageDb, MessageRepo};
use mongodb::bson::{oid::ObjectId, Document};

pub struct MessageService {
    repo: MessageRepo,
}

impl MessageService {
    pub fn new(repo: MessageRepo) -> Self {
        Self { repo }
    }

    pub async fn create_message(
        &self,
        channel_id: String,
        author_id: String,
        content: String,
        created_at: String,
        reply_to: Option<ObjectId>,
        attachment_url: Option<String>,
    ) -> mongodb::error::Result<ObjectId> {
        let msg = MessageDb {
            id: None,
            channel_id,
            author_id,
            content,
            created_at,
            reply_to,
            attachment_url,
            reactions: None,
            pinned_by: None,
            pinned_at: None,
        };

        self.repo.insert(msg).await
    }

    pub async fn get_history(
        &self,
        channel_id: &str,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<MessageDb>, ()> {
        Ok(self.repo.find_by_channel(channel_id, page, per_page).await)
    }

    pub async fn get_history_before(
        &self,
        channel_id: &str,
        before_id: &ObjectId,
        per_page: u64,
    ) -> Result<Vec<MessageDb>, ()> {
        Ok(self.repo.find_by_channel_before(channel_id, before_id, per_page).await)
    }

    pub async fn search_messages(&self, channel_id: &str, q: &str, page: u64, per_page: u64) -> Result<Vec<MessageDb>, ()> {
        Ok(self.repo.search_in_channel(channel_id, q, page, per_page).await)
    }

    pub async fn get_message_by_id(&self, message_id: &ObjectId) -> Option<MessageDb> {
        self.repo.find_by_id(message_id).await
    }

    pub async fn delete_message(&self, message_id: &ObjectId) -> mongodb::error::Result<bool> {
        self.repo.delete(message_id).await
    }

    pub async fn add_reaction(
        &self,
        message_id: &ObjectId,
        emoji: &str,
        user_id: &str,
        username: Option<&str>,
    ) -> mongodb::error::Result<(Option<String>, bool)> {
        self.repo
            .add_reaction(message_id, emoji, user_id, username)
            .await
    }

    pub async fn remove_reaction(
        &self,
        message_id: &ObjectId,
        emoji: &str,
        user_id: &str,
    ) -> mongodb::error::Result<Option<String>> {
        self.repo.remove_reaction(message_id, emoji, user_id).await
    }

    pub async fn edit_message(
        &self,
        message_id: ObjectId,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> mongodb::error::Result<Option<MessageDb>> {
        self
            .repo
            .update_content(message_id, channel_id, author_id, content)
            .await
    }

    pub async fn get_dm_conversations(&self, user_id: &str) -> Vec<Document> {
        self.repo.get_dm_conversations(user_id).await
    }

    pub async fn pin_message(
        &self,
        message_id: ObjectId,
        channel_id: &str,
        pinned_by: &str,
        pinned_at: &str,
    ) -> mongodb::error::Result<Option<MessageDb>> {
        self.repo.pin_message(message_id, channel_id, pinned_by, pinned_at).await
    }

    pub async fn unpin_message(
        &self,
        message_id: ObjectId,
        channel_id: &str,
    ) -> mongodb::error::Result<Option<MessageDb>> {
        self.repo.unpin_message(message_id, channel_id).await
    }

    pub async fn list_pinned(
        &self,
        channel_id: &str,
        page: u64,
        per_page: u64,
    ) -> Vec<MessageDb> {
        self.repo.find_pinned_by_channel(channel_id, page, per_page).await
    }
}
