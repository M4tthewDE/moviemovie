CREATE TABLE movies (
    movie_id SERIAL PRIMARY KEY,
    tmdb_id INT,
    title VARCHAR(255) NOT NULL,
    release_date VARCHAR(255) NOT NULL,
    runtime_minutes INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE persons (
    person_id SERIAL PRIMARY KEY,
    tmdb_id INT,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE movie_cast (
    movie_id INT NOT NULL,
    person_id INT NOT NULL,
    character_name VARCHAR(255),
    PRIMARY KEY (movie_id, person_id, character_name),
    FOREIGN KEY (movie_id) REFERENCES movies(movie_id) ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES persons(person_id) ON DELETE CASCADE
);

CREATE INDEX idx_movie_cast_movie ON movie_cast(movie_id);
CREATE INDEX idx_movie_cast_person ON movie_cast(person_id);
