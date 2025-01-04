// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        content -> Text,
        archived -> Bool,
    }
}
