use std::sync::{Arc, Mutex};

use libcosiauthd::{Authd, Group, User, AuthdClient};
use tarpc::context::Context;

use crate::ProxyConfig;

#[derive(Debug, Clone)]
pub struct ProxySession {
    config: Arc<ProxyConfig>,
    client: Arc<Mutex<AuthdClient>>,
}

#[tarpc::server]
impl Authd for ProxySession {
    async fn get_all_groups(self, _ctx: Context) -> Vec<Group> {
        let client = self.client.lock().unwrap();

        todo!()
    }

    async fn get_group_by_name(self, _ctx: Context, name: String) -> Option<Group> {
        todo!()
    }

    async fn get_group_by_gid(self, _ctx: Context, gid: u32) -> Option<Group> {
        todo!()
    }

    async fn get_all_passwd(self, _ctx: Context) -> Vec<User> {
        todo!()
    }

    async fn get_passwd_by_name(self, _ctx: Context, name: String) -> Option<User> {
        todo!()
    }

    async fn get_passwd_by_uid(self, _ctx: Context, uid: u32) -> Option<User> {
        todo!()
    }
}
