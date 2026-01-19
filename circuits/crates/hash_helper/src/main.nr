use zkpoker_lib::hash_with_salt;

fn main(value: Field, salt: Field) -> pub Field {
    hash_with_salt(value, salt)
}
