-- Your SQL goes here

CREATE TABLE IF NOT EXISTS public.notifications
(
    id bigserial PRIMARY KEY NOT NULL,
    owner_id bigint NOT NULL,
    sender_id bigint,
    subject character varying(255) COLLATE pg_catalog."default" NOT NULL,
    body character varying COLLATE pg_catalog."default" NOT NULL,
    kind character varying(50) COLLATE pg_catalog."default" NOT NULL,
    is_read boolean NOT NULL DEFAULT false,
    extra jsonb NOT NULL,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
)