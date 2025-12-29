-- Миграция для изменения дефолтного значения show_contacts на true

-- Обновляем дефолтное значение для существующих пользователей (опционально)
-- Можно раскомментировать, если нужно обновить всех пользователей:
-- UPDATE users SET show_contacts = true WHERE show_contacts = false;

-- Изменяем дефолтное значение в схеме БД
ALTER TABLE users ALTER COLUMN show_contacts SET DEFAULT true;

