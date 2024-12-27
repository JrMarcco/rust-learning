use crate::pb::User;
use pb::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserReq, GetUserReq,
};
use tonic::{transport::Server, Request, Response, Status};

pub mod pb {
    use prost_types::Timestamp;
    use std::time::SystemTime;

    include!(concat!("../src/pb", "/demo.rs"));

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

pub struct UserServer;

#[tonic::async_trait]
impl UserService for UserServer {
    async fn get_user(&self, req: Request<GetUserReq>) -> Result<Response<User>, Status> {
        let input = req.into_inner();
        println!("input: {:?}", input);
        Ok(Response::new(User::new(
            input.id,
            "jrmaarcco",
            "12345678901",
            "jrmaarcco@example.com",
        )))
    }

    async fn create_user(&self, req: Request<CreateUserReq>) -> Result<Response<User>, Status> {
        let input = req.into_inner();
        println!("input: {:?}", input);
        Ok(Response::new(User::new(
            1,
            &input.name,
            &input.phone,
            &input.email,
        )))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "[::1]:8081".parse()?;
    let svc = UserServer {};

    println!("User Service listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
