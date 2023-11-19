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
