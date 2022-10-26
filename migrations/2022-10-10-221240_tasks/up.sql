CREATE TYPE task_condition AS ENUM ('undone', 'active', 'done');

CREATE TABLE tasks (
    id uuid DEFAULT uuid_generate_v4 (),
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    condition task_condition NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    PRIMARY KEY (id)
)