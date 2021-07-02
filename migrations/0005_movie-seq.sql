BEGIN;

DROP TABLE movie_dates;

CREATE TABLE movie_seq (
    id INTEGER NOT NULL,
    seq SERIAL PRIMARY KEY
);

COMMIT;
