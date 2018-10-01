CREATE TABLE keywords (
	keyword  TEXT PRIMARY KEY,
	owner    BIGINT NOT NULL,
	bareword BOOLEAN NOT NULL,
	hidden   BOOLEAN NOT NULL,
	protect  BOOLEAN NOT NULL,
	shuffle  BOOLEAN NOT NULL
);

CREATE TABLE definitions (
	keyword    TEXT   NOT NULL REFERENCES keywords(keyword),
	definition TEXT   NOT NULL,
	submitter  BIGINT NOT NULL,
	timestamp  TIMESTAMP WITH TIME ZONE NOT NULL,
	embedded   BOOLEAN NOT NULL,

	PRIMARY KEY (keyword, definition)
);
