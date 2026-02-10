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
    entry_info JSONB NOT NULL DEFAULT '{}'::JSONB
);

CREATE TABLE entry_alias (
    old_id VARCHAR(50) NOT NULL PRIMARY KEY,
    new_id VARCHAR(50) NOT NULL REFERENCES entry(id) ON DELETE CASCADE
);
