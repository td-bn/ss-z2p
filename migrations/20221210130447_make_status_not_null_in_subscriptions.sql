-- Add migration script here
BEGIN;
    UPDATE subscriptions
        SET status = 'confirmed'
        where status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status set NOT NULL;
COMMIT;
