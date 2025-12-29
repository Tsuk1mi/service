-- Миграция для добавления поддержки нескольких автомобилей на пользователя

-- Создаём таблицу для хранения множественных автомобилей пользователя
CREATE TABLE IF NOT EXISTS user_plates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plate VARCHAR(20) NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, plate)
);

-- Создаём индексы
CREATE INDEX IF NOT EXISTS idx_user_plates_user_id ON user_plates(user_id);
CREATE INDEX IF NOT EXISTS idx_user_plates_plate ON user_plates(plate);
CREATE INDEX IF NOT EXISTS idx_user_plates_primary ON user_plates(user_id, is_primary) WHERE is_primary = true;

-- Миграция данных: копируем существующие автомобили из users.plate в user_plates
INSERT INTO user_plates (user_id, plate, is_primary, created_at, updated_at)
SELECT id, plate, true, created_at, updated_at
FROM users
WHERE NOT EXISTS (
    SELECT 1 FROM user_plates WHERE user_plates.user_id = users.id AND user_plates.plate = users.plate
);

-- Добавляем колонку для пометки основного автомобиля в таблице users (опционально, для обратной совместимости)
-- Основной автомобиль теперь хранится в user_plates с is_primary = true

