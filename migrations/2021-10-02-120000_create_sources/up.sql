CREATE TABLE sources (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    typ INTEGER NOT NULL,
    url TEXT,
    playlist TEXT,
    headers TEXT,
    enabled INTEGER NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE INDEX idx_sources_playlist
    ON sources (playlist);
CREATE INDEX idx_sources_updated_at
    ON sources (updated_at);

INSERT INTO sources (id, name, typ, url, playlist, headers, enabled) VALUES
    (1, 'Kamikochi', 2, 'https://www.youtube.com/watch?v=Iv2VUE_UhRQ', null, null, 1),
    (2, 'KamniskoSedlo', 1, 'http://pdkamnik.si/watermark.php?filename=sedlo.jpg', null, null, 1),
    (3, 'PlansarskoJezero', 3, null, 'https://livestream.panoramicam.eu/Jezersko/Jezersko.stream/playlist.m3u8', 'Referer: https://panoramicam.eu/', 1);
