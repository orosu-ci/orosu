-- Your SQL goes here
create table workers
(
    id              text                               not null
        constraint workers_pk
            primary key,
    name            text     default ''                not null,
    created_on      datetime default CURRENT_TIMESTAMP not null,
    modified_on     datetime default CURRENT_TIMESTAMP not null,
    secret_key      blob                               not null
);

