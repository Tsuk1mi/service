use crate::error::AppResult;
use sqlx::PgPool;

/// Автоматически создаёт БД и таблицы, если их нет
pub async fn ensure_database_and_tables(pool: &PgPool) -> AppResult<()> {
    tracing::info!("Ensuring database schema exists...");

    // Создаём функцию для автоматического обновления updated_at
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql';
        "#,
    )
    .execute(pool)
    .await?;

    // Создаём таблицу users, если её нет
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            phone_encrypted TEXT,
            phone_hash TEXT,
            telegram TEXT,
            plate TEXT,
            name TEXT,
            show_contacts BOOLEAN NOT NULL DEFAULT true,
            owner_type TEXT DEFAULT 'renter' CHECK (owner_type IN ('owner', 'renter')),
            owner_info JSONB,
            departure_time TIME,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            -- Constraints для валидации данных
            CONSTRAINT telegram_format CHECK (telegram IS NULL OR (LENGTH(telegram) >= 1 AND LENGTH(telegram) <= 32)),
            CONSTRAINT name_length CHECK (name IS NULL OR (LENGTH(TRIM(name)) >= 1 AND LENGTH(name) <= 100)),
            CONSTRAINT plate_format CHECK (plate IS NULL OR (LENGTH(TRIM(plate)) >= 8 AND LENGTH(plate) <= 15))
        )
        "#
    )
    .execute(pool)
    .await?;

    // Гарантируем наличие колонки plate (для старых БД)
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = 'plate'
            ) THEN
                ALTER TABLE users ADD COLUMN plate TEXT;
            END IF;
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = 'phone_hash'
            ) THEN
                ALTER TABLE users ADD COLUMN phone_hash TEXT;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Создаём триггер для автоматического обновления updated_at
    // Сначала удаляем триггер, если он существует
    let _ = sqlx::query(
        r#"
        DROP TRIGGER IF EXISTS update_users_updated_at ON users
        "#,
    )
    .execute(pool)
    .await;

    // Затем создаём новый триггер
    sqlx::query(
        r#"
        CREATE TRIGGER update_users_updated_at
            BEFORE UPDATE ON users
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column()
        "#,
    )
    .execute(pool)
    .await?;

    // Добавляем новые колонки, если их нет (для существующих БД)
    sqlx::query(
        r#"
        DO $$ 
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                          WHERE table_name = 'users' AND column_name = 'owner_type') THEN
                ALTER TABLE users ADD COLUMN owner_type TEXT DEFAULT 'renter' 
                    CHECK (owner_type IN ('owner', 'renter'));
            END IF;
            
            IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                          WHERE table_name = 'users' AND column_name = 'owner_info') THEN
                ALTER TABLE users ADD COLUMN owner_info JSONB;
            END IF;
            
                        IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                                      WHERE table_name = 'users' AND column_name = 'departure_time') THEN
                            ALTER TABLE users ADD COLUMN departure_time TIME;
                        END IF;

                        -- Изменяем plate на nullable, если это еще не сделано
                        IF EXISTS (SELECT 1 FROM information_schema.columns
                                  WHERE table_name = 'users' AND column_name = 'plate' AND is_nullable = 'NO') THEN
                            ALTER TABLE users ALTER COLUMN plate DROP NOT NULL;
                        END IF;
                        
                        -- Обновляем constraint plate_format для поддержки NULL
                        IF EXISTS (SELECT 1 FROM information_schema.constraint_column_usage
                                  WHERE table_name = 'users' AND constraint_name = 'plate_format') THEN
                            ALTER TABLE users DROP CONSTRAINT IF EXISTS plate_format;
                        END IF;
                        ALTER TABLE users ADD CONSTRAINT plate_format 
                            CHECK (plate IS NULL OR (LENGTH(TRIM(plate)) >= 8 AND LENGTH(plate) <= 15));
        END $$;
        "#
    )
    .execute(pool)
    .await?;

    // Создаём таблицу blocks, если её нет
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blocks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            blocker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            blocker_plate TEXT NOT NULL,
            blocked_plate TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            -- Constraints для валидации
            CONSTRAINT blocked_plate_format CHECK (LENGTH(TRIM(blocked_plate)) >= 8 AND LENGTH(blocked_plate) <= 15),
            -- Уникальность: один пользователь не может заблокировать один номер дважды
            CONSTRAINT unique_user_block UNIQUE(blocker_id, blocked_plate)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Создаём составные индексы для оптимизации запросов блокировок
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocks_blocker_id ON blocks(blocker_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocks_blocked_plate ON blocks(blocked_plate)
        "#,
    )
    .execute(pool)
    .await?;

    // Составной индекс для быстрого поиска блокировок по blocker_id и created_at
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocks_blocker_created ON blocks(blocker_id, created_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    // Составной индекс для поиска блокировок по номеру и дате (для проверки блокировки)
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocks_plate_created ON blocks(blocked_plate, created_at DESC)
        "#
    )
    .execute(pool)
    .await?;

    // Гарантируем наличие blocker_plate в blocks (для существующих БД)
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'blocks' AND column_name = 'blocker_plate'
            ) THEN
                ALTER TABLE blocks ADD COLUMN blocker_plate TEXT NOT NULL DEFAULT '';
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Индекс для поиска блокировок по номеру блокирующего
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocks_blocker_plate_norm ON blocks(UPPER(TRIM(blocker_plate)))
        "#
    )
    .execute(pool)
    .await?;

    // Индекс для users.plate с нормализацией (верхний регистр для поиска)
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_plate ON users(UPPER(TRIM(plate)))
        "#,
    )
    .execute(pool)
    .await?;

    // Индекс для оптимизации поиска пользователей по телефону (если используется)
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone_encrypted) WHERE phone_encrypted IS NOT NULL
        "#
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_users_phone_hash_unique ON users(phone_hash) WHERE phone_hash IS NOT NULL
        "#
    )
    .execute(pool)
    .await?;

    // Индекс для поиска по telegram (если используется)
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_telegram ON users(LOWER(telegram)) WHERE telegram IS NOT NULL
        "#
    )
    .execute(pool)
    .await?;

    // Создаём таблицу для множественных автомобилей пользователя
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_plates (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            plate TEXT NOT NULL,
            is_primary BOOLEAN NOT NULL DEFAULT false,
            departure_time TIME,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            -- Constraints
            CONSTRAINT plate_format CHECK (LENGTH(TRIM(plate)) >= 8 AND LENGTH(plate) <= 15),
            UNIQUE(user_id, plate)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Создаём триггер для автоматического обновления updated_at в user_plates
    // Сначала удаляем триггер, если он существует
    let _ = sqlx::query(
        r#"
        DROP TRIGGER IF EXISTS update_user_plates_updated_at ON user_plates
        "#,
    )
    .execute(pool)
    .await;

    // Затем создаём новый триггер
    sqlx::query(
        r#"
        CREATE TRIGGER update_user_plates_updated_at
            BEFORE UPDATE ON user_plates
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column()
        "#,
    )
    .execute(pool)
    .await?;

    // Гарантируем наличие колонки departure_time в user_plates
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'user_plates' AND column_name = 'departure_time'
            ) THEN
                ALTER TABLE user_plates ADD COLUMN departure_time TIME;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Создаём индексы для user_plates
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_user_plates_user_id ON user_plates(user_id)
        "#,
    )
    .execute(pool)
    .await?;

    // Гарантируем наличие push_token в users
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'users' AND column_name = 'push_token'
            ) THEN
                ALTER TABLE users ADD COLUMN push_token TEXT;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_user_plates_plate ON user_plates(plate)
        "#,
    )
    .execute(pool)
    .await?;

    // Уникальный частичный индекс для обеспечения одного основного номера на пользователя
    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_user_plates_primary ON user_plates(user_id) WHERE is_primary = true
        "#
    )
    .execute(pool)
    .await?;

    // Индекс для поиска пользователя по номеру авто (с нормализацией)
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_user_plates_plate_user ON user_plates(UPPER(TRIM(plate)), user_id)
        "#
    )
    .execute(pool)
    .await?;

    // Функциональный индекс для быстрого поиска по номеру (без учета регистра и пробелов)
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_user_plates_plate_normalized ON user_plates(UPPER(TRIM(plate)))
        "#
    )
    .execute(pool)
    .await?;

    // Миграция существующих данных: копируем plate из users в user_plates
    sqlx::query(
        r#"
        INSERT INTO user_plates (user_id, plate, is_primary, created_at, updated_at)
        SELECT id, plate, true, created_at, updated_at
        FROM users
        WHERE plate IS NOT NULL 
          AND LENGTH(TRIM(plate)) > 0
          AND NOT EXISTS (
            SELECT 1 FROM user_plates WHERE user_plates.user_id = users.id AND user_plates.plate = users.plate
        )
        "#
    )
    .execute(pool)
    .await?;

    // Создаём таблицу notifications, если её нет
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notifications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            type TEXT NOT NULL,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            data JSONB,
            read BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            -- Constraints для валидации
            CONSTRAINT notification_type_check CHECK (type IN ('block_created', 'block_deleted', 'warning_call', 'system')),
            CONSTRAINT title_length CHECK (LENGTH(TRIM(title)) >= 1 AND LENGTH(title) <= 200),
            CONSTRAINT message_length CHECK (LENGTH(TRIM(message)) >= 1 AND LENGTH(message) <= 1000)
        )
        "#
    )
    .execute(pool)
    .await?;

    // Создаём индексы для notifications
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at DESC)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_notifications_user_read ON notifications(user_id, read, created_at DESC)
        "#
    )
    .execute(pool)
    .await?;

    // Покрывающий индекс для частого запроса списка уведомлений
    // INCLUDE доступен только в PostgreSQL 11+, игнорируем ошибку если не поддерживается
    let _ = sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_notifications_user_read_covering 
        ON notifications(user_id, read, created_at DESC) 
        INCLUDE (id, type, title, message)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        // Если INCLUDE не поддерживается (старая версия PostgreSQL), логируем и продолжаем
        tracing::warn!(
            "Covering index not supported (PostgreSQL 11+ required), skipping: {}",
            e
        );
        e
    });

    // Индекс для blocks с нормализованным номером
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_blocks_blocked_plate_normalized ON blocks(UPPER(TRIM(blocked_plate)))
        "#
    )
    .execute(pool)
    .await?;

    tracing::info!("Database schema ensured successfully");
    Ok(())
}
