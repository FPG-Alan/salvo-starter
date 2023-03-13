-- Your SQL goes here

CREATE TABLE IF NOT EXISTS public.security_codes
(
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    email character varying(255) COLLATE pg_catalog."default",
    value character varying(255) COLLATE pg_catalog."default" NOT NULL,
    send_method character varying(255) COLLATE pg_catalog."default" NOT NULL,
    consumed_at timestamp with time zone,
    expired_at timestamp with time zone NOT NULL,
    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);