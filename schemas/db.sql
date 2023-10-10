DROP TABLE IF EXISTS awesome_projects CASCADE;

DROP TABLE IF EXISTS awesome_lists CASCADE;

DROP TABLE IF EXISTS awesome_links CASCADE;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


/*
 * MIT License
 *
 * Copyright (c) 2023 Fabio Lima
 * 
 *  Permission is hereby granted, free of charge, to any person obtaining a copy
 *  of this software and associated documentation files (the "Software"), to deal
 *  in the Software without restriction, including without limitation the rights
 *  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 *  copies of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 * 
 *  The above copyright notice and this permission notice shall be included in
 *  all copies or substantial portions of the Software.
 * 
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 *  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 *  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 *  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 *  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 *  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 *  THE SOFTWARE.
 */
/**
 * Returns a time-ordered with Unix Epoch UUID (UUIDv7).
 * 
 * Referencies:
 * - https://github.com/uuid6/uuid6-ietf-draft
 * - https://github.com/ietf-wg-uuidrev/rfc4122bis
 *
 * MIT License.
 *
 * Tags: uuid guid uuid-generator guid-generator generator time order rfc4122 rfc-4122
 */
CREATE OR REPLACE FUNCTION _uuid_generate_v7()
  RETURNS uuid
  AS $$
DECLARE
  v_time timestamp with time zone := NULL;
  v_secs bigint := NULL;
  v_msec bigint := NULL;
  v_usec bigint := NULL;
  v_timestamp bigint := NULL;
  v_timestamp_hex varchar := NULL;
  v_random bigint := NULL;
  v_random_hex varchar := NULL;
  v_bytes bytea;
  c_variant bit(64) := x'8000000000000000';
  -- RFC-4122 variant: b'10xx...'
BEGIN
  -- Get seconds and micros
  v_time := clock_timestamp();
  v_secs := EXTRACT(EPOCH FROM v_time);
  v_msec := mod(EXTRACT(MILLISECONDS FROM v_time)::numeric, 10 ^ 3::numeric);
  v_usec := mod(EXTRACT(MICROSECONDS FROM v_time)::numeric, 10 ^ 3::numeric);
  -- Generate timestamp hexadecimal (and set version 7)
  v_timestamp :=(((v_secs * 10 ^ 3) + v_msec)::bigint << 12) |(v_usec << 2);
  v_timestamp_hex := lpad(to_hex(v_timestamp), 16, '0');
  v_timestamp_hex := substr(v_timestamp_hex, 2, 12) || '7' || substr(v_timestamp_hex, 14, 3);
  -- Generate the random hexadecimal (and set variant b'10xx')
  v_random :=((random()::numeric * 2 ^ 62::numeric)::bigint::bit(64) | c_variant)::bigint;
  v_random_hex := lpad(to_hex(v_random), 16, '0');
  -- Concat timestemp and random hexadecimal
  v_bytes := decode(v_timestamp_hex || v_random_hex, 'hex');
  RETURN encode(v_bytes, 'hex')::uuid;
END
$$
LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS awesome_projects(
  id uuid PRIMARY KEY DEFAULT _uuid_generate_v7(),
  url text NOT NULL,
  UNIQUE (url)
);

CREATE TABLE IF NOT EXISTS awesome_lists(
  id uuid PRIMARY KEY DEFAULT _uuid_generate_v7(),
  project_id uuid NOT NULL REFERENCES awesome_projects(id) ON DELETE CASCADE,
  latest_commit_at timestamp with time zone NOT NULL,
  crawled_at timestamp with time zone NOT NULL,
  UNIQUE (project_id)
);

CREATE TABLE IF NOT EXISTS awesome_links(
  id uuid PRIMARY KEY DEFAULT _uuid_generate_v7(),
  awesome_list_id uuid NOT NULL REFERENCES awesome_lists(id),
  project_id uuid NOT NULL REFERENCES awesome_projects(id) ON DELETE CASCADE,
  title text,
  description text,
  UNIQUE (awesome_list_id, project_id),
  CHECK (awesome_list_id != project_id))
