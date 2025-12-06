-- Your SQL goes here
create table tasks
(
    id          text                               not null
        constraint tasks_pk
            primary key,
    script_key  text                               not null
        constraint tasks_to_scripts_fk
            references scripts (key)
            on delete cascade,
    exit_code   integer,
    output      blob                               not null,
    arguments   blob,
    created_on  datetime default CURRENT_TIMESTAMP not null,
    launched_on datetime,
    finished_on datetime
);

