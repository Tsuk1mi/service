-- Добавляем phone_hash для детерминированного поиска по телефону
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_hash TEXT;

-- Индекс по phone_hash для быстрого поиска (не уникальный для совместимости со старыми дубликатами)
CREATE INDEX IF NOT EXISTS idx_users_phone_hash ON users(phone_hash) WHERE phone_hash IS NOT NULL;

