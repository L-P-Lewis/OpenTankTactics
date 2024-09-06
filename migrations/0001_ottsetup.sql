
create table game (
    game_id char(10) not null,
    admin_id int NOT NULL,
    join_code varchar,
    max_players INT,
    game_layout JSONB
    PRIMARY KEY (game_id),
);

create table player (
    player_id SERIAL,
    player_name varchar NOT NULL,
    passcode char(10) NOT NULL,
    game char(10) NOT NULL,
	PRIMARY KEY (player_id),
    FOREIGN KEY (game) REFERENCES game(game_id)
);

ALTER TABLE game 
ADD FOREIGN KEY (admin_id) REFERENCES player(player_id)