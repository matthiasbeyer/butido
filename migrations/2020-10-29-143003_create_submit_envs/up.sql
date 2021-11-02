--
-- Copyright (c) 2020-2022 science+computing ag and other contributors
--
-- This program and the accompanying materials are made
-- available under the terms of the Eclipse Public License 2.0
-- which is available at https://www.eclipse.org/legal/epl-2.0/
--
-- SPDX-License-Identifier: EPL-2.0
--

-- Your SQL goes here
CREATE TABLE submit_envs (
    id SERIAL PRIMARY KEY NOT NULL,
    submit_id INTEGER REFERENCES submits(id) NOT NULL,
    env_id    INTEGER REFERENCES envvars(id) NOT NULL,

    CONSTRAINT UC_submitid_envid UNIQUE (submit_id, env_id)
)
