-- Track when a user's password last changed so we can invalidate JWTs
-- minted before that point. The auth layer rejects any token whose `iat`
-- claim is older than this column for the user.
--
-- Stored as Unix epoch seconds to match the JWT `iat` numeric type. Default
-- is 0 so existing tokens issued before this column existed remain valid;
-- they get invalidated only on the next password change.
ALTER TABLE users ADD COLUMN password_changed_at INTEGER NOT NULL DEFAULT 0;
