-- This file should undo anything in `up.sql`
ALTER TABLE prefixes RENAME TO guilds;
ALTER TABLE guilds RENAME COLUMN id TO guild_id;
