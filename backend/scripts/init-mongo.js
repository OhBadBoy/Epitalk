// Placeholder: can be used to seed MongoDB collections if needed
db = db.getSiblingDB('chat');
// Example: create an initial message
db.messages.insertOne({ channel_id: 'global', author_id: 'system', content: 'Welcome to chat', created_at: new Date().toISOString() });
