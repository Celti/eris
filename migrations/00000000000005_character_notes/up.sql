CREATE TABLE notes (
	pin     BIGINT NOT NULL,
	name      TEXT NOT NULL,
	note      TEXT NOT NULL,

	FOREIGN KEY (pin)
		REFERENCES characters (pin)
		ON DELETE CASCADE,

	PRIMARY KEY (name, pin)
);
