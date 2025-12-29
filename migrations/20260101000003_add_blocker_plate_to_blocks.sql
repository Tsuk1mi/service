-- Добавляем колонку blocker_plate для хранения номера блокирующего авто
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS blocker_plate TEXT NOT NULL DEFAULT '';

-- Индекс для поиска по номеру блокирующего
CREATE INDEX IF NOT EXISTS idx_blocks_blocker_plate_norm ON blocks(UPPER(TRIM(blocker_plate)));

