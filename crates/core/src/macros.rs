macro_rules! named_key {
    () => {
        fn key(item: &Self::Item) -> String {
            item.to_string()
        }
    };
}

macro_rules! single_key {
    () => {
        fn key(_item: &$crate::Single) -> String {
            Self::ID.to_owned()
        }
    };
}
