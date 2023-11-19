create table logs (
    "id" SERIAL,
    "level" varchar,
    "message" text,
    "resource_id" varchar,
    "timestamp" timestamptz,
    "trace_id" varchar,
    "span_id" varchar,
    "commit" varchar,
    "parent_resource_id" varchar
);

create index if not exists idx_level on logs("level");
create index if not exists idx_message on logs("message");
create index if not exists idx_resource_id on logs("resource_id");
create index if not exists idx_timestamp on logs("timestamp");
create index if not exists idx_trace_id on logs("trace_id");
create index if not exists idx_span_id on logs("span_id");
create index if not exists idx_commit on logs("commit");
create index if not exists idx_parent_resource_id on logs("parent_resource_id");
