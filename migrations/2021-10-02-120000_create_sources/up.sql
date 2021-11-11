CREATE TABLE sources (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    typ INTEGER NOT NULL,
    url TEXT NOT NULL UNIQUE,
    playlist TEXT,
    enabled INTEGER NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_sources_playlist
    ON sources (playlist);
CREATE INDEX idx_sources_updated_at
    ON sources (updated_at);

INSERT INTO sources (id, name, typ, url, enabled) VALUES
    (1, 'Kamikochi', 2, 'https://www.youtube.com/watch?v=9-sfWSHtJdk', 1),
    (2, 'KamniskoSedlo', 1, 'http://pdkamnik.si/watermark.php?filename=sedlo.jpg', 1);
