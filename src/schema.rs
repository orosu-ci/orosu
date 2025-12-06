// @generated automatically by Diesel CLI.

diesel::table! {
    scripts (rowid) {
        rowid -> Integer,
        key -> Text,
        path -> Text,
        created_on -> Timestamp,
        updated_on -> Timestamp,
        deleted_on -> Nullable<Timestamp>,
        status -> Text,
    }
}

diesel::table! {
    tasks (id) {
        id -> Text,
        script_key -> Text,
        exit_code -> Nullable<Integer>,
        output -> Binary,
        arguments -> Nullable<Binary>,
        created_on -> Timestamp,
        launched_on -> Nullable<Timestamp>,
        finished_on -> Nullable<Timestamp>,
    }
}

diesel::table! {
    workers (id) {
        id -> Text,
        name -> Text,
        created_on -> Timestamp,
        modified_on -> Timestamp,
        secret_key -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(scripts, tasks, workers,);
