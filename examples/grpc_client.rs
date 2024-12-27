use anyhow::Result;
use pb::{user_service_client::UserServiceClient, CreateUserReq, GetUserReq};

pub mod pb {
    include!(concat!("../src/pb", "/demo.rs"));
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:8081").await?;

    let req = GetUserReq { id: 1 };
    let rsp = client.get_user(req).await?;
    println!("resp: {:?}", rsp);

    let req = CreateUserReq {
        name: "test".to_string(),
        phone: "12345678901".to_string(),
        email: "test@test.com".to_string(),
    };
    let rsp = client.create_user(req).await?;
    println!("resp: {:?}", rsp);

    Ok(())
}
