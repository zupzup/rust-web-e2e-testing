CREATE TABLE IF NOT EXISTS todo
(
    id SERIAL PRIMARY KEY NOT NULL,
    name TEXT,
    checked boolean DEFAULT false
);
