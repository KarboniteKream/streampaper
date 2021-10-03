CREATE TABLE images (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,

    FOREIGN KEY (source_id) REFERENCES sources (id)
);

CREATE INDEX idx_images_timestamp_source_id
    ON images (timestamp, source_id);
