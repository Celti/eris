-- Your SQL goes here
CREATE TABLE seen (
	id			BIGINT PRIMARY KEY,
	at			TIMESTAMP WITH TIME ZONE NOT NULL,
	kind		TEXT NOT NULL,
	name		TEXT NOT NULL
)
