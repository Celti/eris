CREATE TYPE activity_kind AS ENUM ('playing', 'listening', 'streaming');

CREATE TABLE bot (
	id             BIGINT PRIMARY KEY,
	activity_type  activity_kind NOT NULL,
	activity_name  TEXT NOT NULL
);
