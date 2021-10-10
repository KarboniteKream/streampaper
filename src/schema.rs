table! {
    images (id) {
        id -> BigInt,
        source_id -> BigInt,
        timestamp -> BigInt,
    }
}

table! {
    sources (id) {
        id -> BigInt,
        name -> Text,
        typ -> Integer,
        url -> Text,
        playlist -> Nullable<Text>,
        enabled -> Bool,
        updated_at -> BigInt,
    }
}

joinable!(images -> sources (source_id));

allow_tables_to_appear_in_same_query!(images, sources,);
