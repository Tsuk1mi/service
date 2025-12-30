-- Изменение блокировок: привязка к номеру машины вместо аккаунта
-- Удаляем старый constraint
ALTER TABLE blocks DROP CONSTRAINT IF EXISTS unique_user_block;

-- Добавляем новый constraint: один номер не может заблокировать другой номер дважды
ALTER TABLE blocks ADD CONSTRAINT unique_plate_block 
    UNIQUE(UPPER(TRIM(blocker_plate)), UPPER(TRIM(blocked_plate)));

-- Создаём индекс для быстрой проверки блокировок по номерам
CREATE INDEX IF NOT EXISTS idx_blocks_blocker_blocked_plates 
    ON blocks(UPPER(TRIM(blocker_plate)), UPPER(TRIM(blocked_plate)));

