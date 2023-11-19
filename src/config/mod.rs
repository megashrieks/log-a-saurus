use std::env;


pub fn get_env(key: &str) -> String {
    for (k, value) in env::vars() {
        if k.eq(key) {
            return value;
        }
    }
    panic!("env not found {}", key);
}
