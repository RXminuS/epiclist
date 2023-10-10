DROP TABLE IF EXISTS awesome_projects CASCADE;

DROP TABLE IF EXISTS awesome_lists CASCADE;

DROP TABLE IF EXISTS awesome_links CASCADE;

CREATE TABLE IF NOT EXISTS awesome_projects(
  id bigserial PRIMARY KEY,
  url text NOT NULL,
  UNIQUE (url)
);

CREATE TABLE IF NOT EXISTS awesome_lists(
  id bigserial PRIMARY KEY,
  project_id integer NOT NULL REFERENCES awesome_projects(id) ON DELETE CASCADE,
  updated_at timestamp without time zone NOT NULL,
  UNIQUE (project_id)
);

CREATE TABLE IF NOT EXISTS awesome_links(
  id bigserial PRIMARY KEY,
  awesome_list_id integer NOT NULL REFERENCES awesome_lists(id),
  project_id integer NOT NULL REFERENCES awesome_projects(id) ON DELETE CASCADE,
  title text,
  description text,
  UNIQUE (awesome_list_id, project_id),
  CHECK (awesome_list_id != project_id))
