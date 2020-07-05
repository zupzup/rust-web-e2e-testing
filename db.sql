CREATE TABLE IF NOT EXISTS todo
(
    id SERIAL PRIMARY KEY NOT NULL,
    name TEXT,
    checked boolean DEFAULT false
);

INSERT INTO todo (id, name, checked) VALUES (10000, 'Get a Cat', 'true') ON CONFLICT DO NOTHING;
INSERT INTO todo (id, name, checked) VALUES (20000, 'Get a Dog', 'false') ON CONFLICT DO NOTHING;
INSERT INTO todo (id, name, checked) VALUES (30000, 'Feed the Cat', 'true') ON CONFLICT DO NOTHING;

