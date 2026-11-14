use env_decorators::{env_type, env_value};

#[test]
fn absent_configuration_keeps_default_type_and_value() {
    type Value = env_type!("DOUBLETS_TEST_UNSET_DECORATORS_68", "<*>", u32);
    let value: Value = env_value!("DOUBLETS_TEST_UNSET_DECORATORS_68", 42);
    assert_eq!(value, 42_u32);
}
