use crate::pb::User;
use anyhow::Result;
use prost::Message;

pub mod pb {
    use prost_types::Timestamp;
    use std::time::SystemTime;

    include!(concat!("pb", "/demo.rs"));

    impl User {
        pub fn new(id: u64, name: &str, phone: &str, email: &str) -> Self {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            let timestamp = Some(Timestamp {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            });

            Self {
                id,
                name: name.to_string(),
                phone: phone.to_string(),
                email: email.to_string(),
                created_at: timestamp,
                updated_at: timestamp,
            }
        }
    }
}

fn main() -> Result<()> {
    let user = User::new(1, "jrmaarcco", "1234567890", "jrmaarcco@example.com");
    let encoded = user.encode_to_vec();
    println!("user: {:?}, encoded: {:?}", user, encoded);

    let decoded: User = User::decode(&encoded[..])?;
    println!("decoded: {:?}", decoded);

    Ok(())
}
