CREATE TABLE sources (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL UNIQUE,
    playlist TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_sources_playlist
    ON sources (playlist);
CREATE INDEX idx_sources_updated_at
    ON sources (updated_at);

INSERT INTO sources (name, url) VALUES
    ('Kamikochi', 'https://www.youtube.com/watch?v=9-sfWSHtJdk'),
    ('Shibuya', 'https://www.youtube.com/watch?v=HpdO5Kq3o7Y');