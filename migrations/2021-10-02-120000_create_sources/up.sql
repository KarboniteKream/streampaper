CREATE TABLE sources (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    typ INTEGER NOT NULL,
    url TEXT,
    playlist TEXT,
    enabled INTEGER NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_sources_playlist
    ON sources (playlist);
CREATE INDEX idx_sources_updated_at
    ON sources (updated_at);

INSERT INTO sources (id, name, typ, url, playlist, enabled) VALUES
    (1, 'Kamikochi', 2, 'https://www.youtube.com/watch?v=9-sfWSHtJdk', null, 1),
    (2, 'KamniskoSedlo', 1, 'http://pdkamnik.si/watermark.php?filename=sedlo.jpg', null, 1),
    (3, 'PlansarskoJezero', 3, null, 'https://stream.pikacom.com/Jezersko/Jezersko.stream/playlist.m3u8', 1);
