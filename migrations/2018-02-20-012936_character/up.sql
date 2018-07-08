CREATE TABLE characters (
	char_id  SERIAL UNIQUE NOT NULL,
	pinned   BIGINT UNIQUE,
	mtime    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	name     TEXT NOT NULL,
	game     TEXT NOT NULL,

	PRIMARY KEY (name, game)
);

CREATE TABLE char_base (
	char_id    INTEGER NOT NULL REFERENCES characters (char_id),
	cur_hp     INTEGER NOT NULL,
	max_hp     INTEGER NOT NULL,
	cur_rp     INTEGER NOT NULL,
	max_rp     INTEGER NOT NULL,
	cur_fp     INTEGER NOT NULL,
	max_fp     INTEGER NOT NULL,
	cur_lfp    INTEGER NOT NULL,
	max_lfp    INTEGER NOT NULL,
	cur_sp     INTEGER NOT NULL,
	max_sp     INTEGER NOT NULL,
	cur_lsp    INTEGER NOT NULL,
	max_lsp    INTEGER NOT NULL,
	cur_ip     INTEGER NOT NULL,
	max_ip     INTEGER NOT NULL,
	xp         INTEGER NOT NULL,

	PRIMARY KEY (char_id)
);

CREATE TABLE char_notes (
	char_id    INTEGER NOT NULL REFERENCES characters (char_id),
	key        TEXT    NOT NULL,
	value      TEXT    NOT NULL,

	PRIMARY KEY (char_id, key)
);

CREATE TABLE char_points (
	char_id    INTEGER NOT NULL REFERENCES characters (char_id),
	maximum    INTEGER NOT NULL,
	value      INTEGER NOT NULL,
	key        TEXT    NOT NULL,

	PRIMARY KEY (char_id, key)
);
