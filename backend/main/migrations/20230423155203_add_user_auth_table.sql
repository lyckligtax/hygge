-- Add migration script here

CREATE TYPE account_status AS ENUM ('active', 'inactive', 'removed');
create table account
(
    id          uuid      default gen_random_uuid(),
    id_external varchar   not null,
    hash        varchar   not null,
    status      account_status default 'active' not null,
    created_at  timestamp not null default NOW(),
    updated_at  timestamp not null default NOW()
);

alter table account add constraint account_pk primary key (id);
alter table account add constraint account_pk2 unique (id_external);

call create_updated_at_trigger('account');