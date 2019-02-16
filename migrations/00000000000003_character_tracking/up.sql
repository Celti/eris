CREATE TABLE characters (
	name      TEXT NOT NULL,
	channel BIGINT NOT NULL,
	owner   BIGINT NOT NULL,
	pin     BIGINT NOT NULL,

	UNIQUE (name, channel),
	PRIMARY KEY (pin)
);


CREATE TABLE attributes (
	pin     BIGINT NOT NULL,
	name      TEXT NOT NULL,
	value      INT NOT NULL,
	maximum    INT NOT NULL,

	FOREIGN KEY (pin)
		REFERENCES characters (pin)
		ON DELETE CASCADE,

	PRIMARY KEY (name, pin)
);
