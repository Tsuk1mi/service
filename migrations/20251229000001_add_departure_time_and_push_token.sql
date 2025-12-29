-- Add departure_time to user_plates
ALTER TABLE user_plates
    ADD COLUMN IF NOT EXISTS departure_time TIME;

-- Add push_token to users for push notifications
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS push_token TEXT;

