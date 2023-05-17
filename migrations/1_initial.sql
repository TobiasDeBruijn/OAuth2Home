CREATE TABLE pending_authorizations (
    authorization_code TEXT NOT NULL,
    client_id TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    expires_at INT NOT NULL,
    PRIMARY KEY (authorization_code)
);

CREATE TABLE authorizations (
    authorization_id TEXT NOT NULL,
    client_id TEXT NOT NULL,
    client_secret TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    PRIMARY KEY (authorization_id),
    UNIQUE (refresh_token)
);

CREATE TABLE access_tokens (
    access_token TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    authorization_id TEXT NOT NULL,
    PRIMARY KEY (access_token),
    FOREIGN KEY (authorization_id) REFERENCES authorizations(authorization_id)
);