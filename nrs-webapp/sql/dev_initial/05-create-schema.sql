CREATE TYPE ENTRYTYPE AS ENUM (
    'Anime',
    'Manga',
    'LightNovel',
    'VisualNovel',
    'MusicArtist',
    'MusicAlbum',
    'MusicTrack',
    'MusicAlbumTrack',
    'Franchise',
    'Game',
    'Other',
    'GenericPerson',
    'GenericOrganization'
);

CREATE TABLE entry (
    id VARCHAR(50) NOT NULL PRIMARY KEY,
    title VARCHAR(512) NOT NULL,
    entry_type ENTRYTYPE NOT NULL,
    added_by UUID REFERENCES app_user(id),
    -- TODO: this is temporary only
    overall_score DOUBLE PRECISION DEFAULT 0,
    entry_info JSONB NOT NULL DEFAULT '{}'::JSONB
);

CREATE INDEX idx_entry_overall_score_desc ON entry (overall_score DESC);
CREATE INDEX idx_entry_type_score_desc ON entry (entry_type, overall_score DESC);
CREATE INDEX idx_entry_non_franchise_score_desc ON entry (overall_score DESC) WHERE entry_type != 'Franchise';
