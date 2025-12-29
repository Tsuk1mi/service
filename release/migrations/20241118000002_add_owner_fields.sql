-- Миграция для добавления полей owner_type и owner_info в таблицу users

-- Добавляем колонку owner_type если её нет
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'owner_type'
    ) THEN
        ALTER TABLE users ADD COLUMN owner_type VARCHAR(20);
    END IF;
END $$;

-- Добавляем колонку owner_info если её нет
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'owner_info'
    ) THEN
        ALTER TABLE users ADD COLUMN owner_info JSONB;
    END IF;
END $$;

