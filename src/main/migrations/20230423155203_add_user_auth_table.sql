-- Add migration script here
create table auth
(
    id          uuid      default gen_random_uuid(),
    id_external varchar   not null,
    hash        varchar   not null,
    status      varchar   default 'active' not null,
    created_at  timestamp not null default NOW(),
    updated_at  timestamp not null default NOW()
);

alter table auth add constraint auth_pk primary key (id);
alter table auth add constraint auth_pk2 unique (id_external);

call create_updated_at_trigger('auth');