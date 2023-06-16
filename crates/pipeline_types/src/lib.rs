pub mod audio;
pub mod materials;
pub mod models;

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_true(value: &bool) -> bool {
    *value
}

fn true_value() -> bool {
    true
}

fn is_default<T: PartialEq + Default>(value: &T) -> bool {
    *value == Default::default()
}
