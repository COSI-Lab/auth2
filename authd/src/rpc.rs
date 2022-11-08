use std::sync::Arc;

use tarpc::context::Context;

use crate::types::{Config, Group, User};

#[tarpc::service]
pub trait Authd {
    async fn get_all_groups() -> Vec<Group>;
    async fn get_group_by_name(name: String) -> Option<Group>;
    async fn get_group_by_gid(gid: u32) -> Option<Group>;

    async fn get_all_passwd() -> Vec<User>;
    async fn get_passwd_by_name(name: String) -> Option<User>;
    async fn get_passwd_by_uid(uid: u32) -> Option<User>;
}

#[derive(Debug, Clone)]
pub struct AuthdSession {
    pub config: Arc<Config>,
}

impl AuthdSession {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[tarpc::server]
impl Authd for AuthdSession {
    async fn get_all_groups(self, _ctx: Context) -> Vec<Group> {
        self.config.groups.clone()
    }

    async fn get_group_by_name(self, _ctx: Context, name: String) -> Option<Group> {
        self.config
            .groups
            .iter()
            .find(|group| group.name == name)
            .map(Group::clone)
    }

    async fn get_group_by_gid(self, _ctx: Context, gid: u32) -> Option<Group> {
        self.config
            .groups
            .iter()
            .find(|group| group.gid == gid)
            .map(Group::clone)
    }

    async fn get_all_passwd(self, _ctx: Context) -> Vec<User> {
        self.config.users.clone()
    }

    async fn get_passwd_by_name(self, _ctx: Context, name: String) -> Option<User> {
        self.config
            .users
            .iter()
            .find(|user| user.name == name)
            .map(User::clone)
    }

    async fn get_passwd_by_uid(self, _ctx: Context, uid: u32) -> Option<User> {
        self.config
            .users
            .iter()
            .find(|user| user.id == uid)
            .map(User::clone)
    }
}
