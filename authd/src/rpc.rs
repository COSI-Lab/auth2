use std::sync::Arc;

use libnss::{group::Group, passwd::Passwd};
use tarpc::context::Context;

use crate::types::{Shell, Config};

#[tarpc::service]
pub trait Authd {
    async fn get_all_groups() -> Vec<Group>;
    async fn get_group_by_name(name: String) -> Option<Group>;
    async fn get_group_by_gid(gid: u32) -> Option<Group>;

    async fn get_all_passwd() -> Vec<Passwd>;
    async fn get_passwd_by_name(name: String) -> Option<Passwd>;
    async fn get_passwd_by_uid(uid: u32) -> Option<Passwd>;

    async fn set_home_root(root: String);
    async fn set_supported_shells(shells: Vec<Shell>);
}

pub struct AuthdSession {
    pub config: Arc<Config>,
    pub home_root: Option<String>,
    pub supported_shells: Option<Vec<Shell>>,
}

impl AuthdSession {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            home_root: None,
            supported_shells: None,
        }
    }
}


#[tarpc::server]
impl Authd for AuthdSession {
    async fn get_all_groups(self, _ctx: Context) -> Vec<Group> {
        self.config
            .groups
            .iter()
            .map(|(k, g)| g.to_nss(k))
            .collect()
    }

    async fn get_group_by_name(self, _ctx: Context, name: String) -> Option<Group> {
        self.config
            .groups
            .iter()
            .find(|(n, _)| **n == name)
            .map(|(n, g)| g.to_nss(n))
    }

    async fn get_group_by_gid(self, _ctx: Context, gid: u32) -> Option<Group> {
        self.config
            .groups
            .iter()
            .find(|(_, g)| g.gid == gid)
            .map(|(n, g)| g.to_nss(n))
    }

    async fn get_all_passwd(self, _ctx: Context) -> Vec<Passwd> {
        let home_root = self.home_root.unwrap_or("/home".to_string());

        self.config
            .users
            .iter()
            .map(|(k, p)| p.to_nss(k, &home_root, &self.supported_shells))
            .collect()
    }

    async fn get_passwd_by_name(self, _ctx: Context, name: String) -> Option<Passwd> {
        let home_root = self.home_root.unwrap_or("/home".to_string());

        self.config
            .users
            .iter()
            .find(|(n, _)| **n == name)
            .map(|(k, p)| p.to_nss(k, &home_root, &self.supported_shells))
    }

    async fn get_passwd_by_uid(self, _ctx: Context, uid: u32) -> Option<Passwd> {
        let home_root = self.home_root.unwrap_or("/home".to_string());

        self.config
            .users
            .iter()
            .find(|(_, p)| p.id == uid)
            .map(|(k, p)| p.to_nss(k, &home_root, &self.supported_shells))
    }

    async fn set_home_root(mut self, _ctx: Context, home_root: String) {
        self.home_root = Some(home_root);
    }

    async fn set_supported_shells(mut self, _ctx: Context, supported_shells: Vec<Shell>) {
        self.supported_shells = Some(supported_shells);
    }
}
