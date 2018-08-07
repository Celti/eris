-- Your SQL goes here
ALTER TABLE guilds RENAME TO prefixes;
ALTER TABLE prefixes RENAME COLUMN guild_id TO id;
