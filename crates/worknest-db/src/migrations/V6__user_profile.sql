-- User profile fields needed by the new dashboard / sidebar.
--
-- The new design shows display names alongside avatars. Until now the
-- `users` table only stored a username (the login handle). These two
-- nullable columns let us add real names + image URLs without breaking
-- existing rows or auth flows.

ALTER TABLE users ADD COLUMN full_name TEXT;
ALTER TABLE users ADD COLUMN avatar_url TEXT;
