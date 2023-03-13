-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.access_tokens
(
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    name character varying(255) COLLATE pg_catalog."default",
    kind character varying(255) COLLATE pg_catalog."default" NOT NULL,
    value character varying(255) COLLATE pg_catalog."default" NOT NULL,
    device character varying(255) COLLATE pg_catalog."default",

    expired_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);