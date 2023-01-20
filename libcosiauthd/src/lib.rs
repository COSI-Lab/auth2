mod socketname;
mod types;

pub use types::*;
pub use socketname::{SocketName, SocketNameError};

#[tarpc::service]
pub trait Authd {
    async fn get_all_groups() -> Vec<Group>;
    async fn get_group_by_name(name: String) -> Option<Group>;
    async fn get_group_by_gid(gid: u32) -> Option<Group>;

    async fn get_all_passwd() -> Vec<User>;
    async fn get_passwd_by_name(name: String) -> Option<User>;
    async fn get_passwd_by_uid(uid: u32) -> Option<User>;
}

