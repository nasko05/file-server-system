CREATE TABLE IF NOT EXISTS users (
                                     id SERIAL PRIMARY KEY,
                                     username VARCHAR(50) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL
    );


INSERT INTO users (username, password_hash, role)
VALUES ('admin', '$2a$12$fMy8PIhmmTAaNanbf4CXIeL/r/XeIhnWfRlgM/fHUykOwQyabT2Ee', 'admin')
RETURNING id;

INSERT INTO users (username, password_hash, role)
VALUES ('adonev', '$2a$12$Qg2Q.ZRPBRJka7/E9h6aP.LCR./QDmAPjMI4IHaf5vj6sWWT6dLeW', 'adonev')
RETURNING id;

SELECT * FROM users;

TRUNCATE users;

create table if not exists privilege_level (
                                               role VARCHAR(50) PRIMARY KEY,
    privelege_level SMALLINT NOT NULL
    );

INSERT INTO privilege_level (role, privelege_level) VALUES ('admin', 999);

SELECT * FROM privilege_level
