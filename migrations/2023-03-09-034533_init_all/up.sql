CREATE TABLE IF NOT EXISTS public.users 
(
    id bigserial PRIMARY KEY NOT NULL,
    ident_name character varying(255) COLLATE pg_catalog."default" NOT NULL,
    display_name character varying(255) COLLATE pg_catalog."default" NOT NULL,
    password character varying(255) COLLATE pg_catalog."default" NOT NULL,
    
    is_disabled boolean NOT NULL DEFAULT false,
    disabled_by bigint,
    disabled_at timestamp with time zone,

    is_verified boolean NOT NULL DEFAULT false,
    verified_at timestamp with time zone,

    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE IF NOT EXISTS public.emails
(
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    value character varying(255) COLLATE pg_catalog."default" NOT NULL,
    domain character varying(255) COLLATE pg_catalog."default" NOT NULL,
    is_verified boolean NOT NULL DEFAULT false,

    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE IF NOT EXISTS public.user_friends
(
    id bigserial PRIMARY KEY NOT NULL,
    user_id bigint NOT NULL,
    firend_id bigint NOT NULL,

    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE IF NOT EXISTS public.messages
(
    id bigserial PRIMARY KEY NOT NULL,
    sender_id bigint NOT NULL,
    recivier_id bigint NOT NULL,
    kind character varying(50) COLLATE pg_catalog."default" NOT NULL DEFAULT '_'::character varying,
    content json NOT NULL DEFAULT '{}'::jsonb,

    updated_by bigint,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by bigint,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);
