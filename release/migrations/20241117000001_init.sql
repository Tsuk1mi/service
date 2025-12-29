-- Создание таблицы пользователей
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_encrypted TEXT,
    telegram TEXT,
    plate VARCHAR(20) NOT NULL,
    name VARCHAR(100),
    show_contacts BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Создание таблицы блокировок
CREATE TABLE IF NOT EXISTS blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blocker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_plate VARCHAR(20) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Индексы для оптимизации запросов
CREATE INDEX IF NOT EXISTS idx_users_phone_encrypted ON users(phone_encrypted);
CREATE INDEX IF NOT EXISTS idx_users_plate ON users(plate);
CREATE INDEX IF NOT EXISTS idx_blocks_blocked_plate ON blocks(blocked_plate);
CREATE INDEX IF NOT EXISTS idx_blocks_blocker_id ON blocks(blocker_id);
CREATE INDEX IF NOT EXISTS idx_blocks_created_at ON blocks(created_at DESC);

