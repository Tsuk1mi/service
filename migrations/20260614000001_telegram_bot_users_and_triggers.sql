-- Функция автообновления updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Таблица связей Telegram бота (phone_hash <-> chat_id)
CREATE TABLE IF NOT EXISTS telegram_bot_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_hash TEXT NOT NULL,
    chat_id BIGINT NOT NULL,
    telegram_username TEXT,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(phone_hash, chat_id)
);

CREATE INDEX IF NOT EXISTS idx_telegram_bot_users_phone_hash ON telegram_bot_users(phone_hash);
CREATE INDEX IF NOT EXISTS idx_telegram_bot_users_chat_id ON telegram_bot_users(chat_id);
CREATE INDEX IF NOT EXISTS idx_telegram_bot_users_user_id ON telegram_bot_users(user_id) WHERE user_id IS NOT NULL;

DROP TRIGGER IF EXISTS update_telegram_bot_users_updated_at ON telegram_bot_users;
CREATE TRIGGER update_telegram_bot_users_updated_at
    BEFORE UPDATE ON telegram_bot_users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Триггеры updated_at для основных таблиц
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_user_plates_updated_at ON user_plates;
CREATE TRIGGER update_user_plates_updated_at
    BEFORE UPDATE ON user_plates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- departure_time в user_plates
ALTER TABLE user_plates ADD COLUMN IF NOT EXISTS departure_time TIME;

-- push_token в users
ALTER TABLE users ADD COLUMN IF NOT EXISTS push_token TEXT;
