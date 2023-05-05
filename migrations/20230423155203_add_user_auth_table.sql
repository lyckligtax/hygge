-- Add migration script here
create table auth
(
    id          uuid,
    external_id varchar   not null,
    hash        varchar   not null,
    created_at  timestamp not null default NOW(),
    updated_at  timestamp not null default NOW()
);

alter table auth add constraint auth_pk primary key (id);
alter table auth add constraint auth_pk2 unique (external_id);

call create_updated_at_trigger('auth');