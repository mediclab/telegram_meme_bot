ALTER TABLE memes ADD long_hash VARCHAR(256);
ALTER TABLE memes ADD short_hash VARCHAR(4);

CREATE INDEX ON memes (short_hash);