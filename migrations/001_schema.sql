CREATE TABLE IF NOT EXISTS series (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    year INTEGER,
    thumbnail TEXT,
    backdrop TEXT,
    trailer_url TEXT,
    genres TEXT,
    status TEXT DEFAULT 'ongoing',
    views INTEGER DEFAULT 0,
    featured INTEGER DEFAULT 0,
    rating REAL DEFAULT 8.5,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    season_number INTEGER NOT NULL,
    title TEXT,
    description TEXT,
    trailer_url TEXT,
    created_at TEXT,
    updated_at TEXT,
    UNIQUE(series_id, season_number)
);

CREATE TABLE IF NOT EXISTS episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    season_id INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    episode_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    thumbnail TEXT,
    video_source_primary TEXT,
    video_source_backup TEXT,
    download_link TEXT,
    size_1080p TEXT,
    download_bluray TEXT,
    size_bluray TEXT,
    download_720p TEXT,
    size_720p TEXT,
    download_480p TEXT,
    size_480p TEXT,
    duration INTEGER,
    views INTEGER DEFAULT 0,
    created_at TEXT,
    updated_at TEXT,
    UNIQUE(season_id, episode_number)
);

CREATE TABLE IF NOT EXISTS subtitles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    episode_id INTEGER NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
    language TEXT DEFAULT 'urdu',
    label TEXT,
    file_path TEXT NOT NULL,
    is_default INTEGER DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS admins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    name TEXT,
    email TEXT,
    remember_token TEXT,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS site_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_name TEXT DEFAULT 'Turkish Times',
    logo_path TEXT,
    favicon_path TEXT,
    description TEXT,
    created_at TEXT,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_series_title ON series(title);
CREATE INDEX IF NOT EXISTS idx_series_created ON series(created_at);
CREATE INDEX IF NOT EXISTS idx_series_views ON series(views);
CREATE INDEX IF NOT EXISTS idx_series_status_featured ON series(status, featured);
CREATE INDEX IF NOT EXISTS idx_episodes_season ON episodes(season_id);
CREATE INDEX IF NOT EXISTS idx_episodes_created ON episodes(created_at);
CREATE INDEX IF NOT EXISTS idx_seasons_series ON seasons(series_id);
CREATE INDEX IF NOT EXISTS idx_subtitles_episode ON subtitles(episode_id);
