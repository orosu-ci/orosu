-- Your SQL goes here
create table scripts
(
    key        text                               not null,
    path       text                               not null,
    created_on datetime default CURRENT_TIMESTAMP not null,
    updated_on datetime default CURRENT_TIMESTAMP not null,
    deleted_on datetime,
    status     text     default 'ACTIVE'          not null
);

