-- Your SQL goes here

ALTER TABLE IF EXISTS public.users
    ADD COLUMN in_kernel boolean NOT NULL DEFAULT false;