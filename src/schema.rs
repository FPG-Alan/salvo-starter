// @generated automatically by Diesel CLI.

diesel::table! {
    access_tokens (id) {
        id -> Int8,
        user_id -> Int8,
        name -> Nullable<Varchar>,
        kind -> Varchar,
        value -> Varchar,
        device -> Nullable<Varchar>,
        expired_at -> Timestamptz,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    emails (id) {
        id -> Int8,
        user_id -> Int8,
        value -> Varchar,
        domain -> Varchar,
        is_verified -> Bool,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    messages (id) {
        id -> Int8,
        sender_id -> Int8,
        recivier_id -> Int8,
        kind -> Varchar,
        content -> Json,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    notifications (id) {
        id -> Int8,
        owner_id -> Int8,
        sender_id -> Nullable<Int8>,
        subject -> Varchar,
        body -> Varchar,
        kind -> Varchar,
        is_read -> Bool,
        extra -> Jsonb,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    security_codes (id) {
        id -> Int8,
        user_id -> Int8,
        email -> Nullable<Varchar>,
        value -> Varchar,
        send_method -> Varchar,
        consumed_at -> Nullable<Timestamptz>,
        expired_at -> Timestamptz,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_friends (id) {
        id -> Int8,
        user_id -> Int8,
        firend_id -> Int8,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        ident_name -> Varchar,
        display_name -> Varchar,
        password -> Varchar,
        is_disabled -> Bool,
        disabled_by -> Nullable<Int8>,
        disabled_at -> Nullable<Timestamptz>,
        is_verified -> Bool,
        verified_at -> Nullable<Timestamptz>,
        updated_by -> Nullable<Int8>,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int8>,
        created_at -> Timestamptz,
        in_kernel -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    access_tokens,
    emails,
    messages,
    notifications,
    security_codes,
    user_friends,
    users,
);
