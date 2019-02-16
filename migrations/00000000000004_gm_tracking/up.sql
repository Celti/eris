CREATE TABLE channels (
	channel BIGINT NOT NULL,
	gm      BIGINT NOT NULL,

	UNIQUE (channel, gm),
	PRIMARY KEY (channel)
);
