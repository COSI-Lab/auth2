use std::collections::HashMap;

use libcosiauthd::{Group, User};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub groups: Vec<Group>,
    pub users: Vec<User>,
    pub cert: String,
    pub key: String,
}

impl Config {
    /// Verifies all defined ids and gids have no overlap.
    /// Prints the first reported error to stderr
    pub(crate) fn check_dup(&self) -> bool {
        enum GroupOrUser<'a> {
            User(&'a String, u32),
            Group(&'a String, u32),
        }
        impl GroupOrUser<'_> {
            fn get_id(&self) -> u32 {
                match self {
                    GroupOrUser::User(_, id) => *id,
                    GroupOrUser::Group(_, gid) => *gid,
                }
            }
        }

        let mut reverse: HashMap<u32, GroupOrUser> = HashMap::new();

        let groups = self
            .groups
            .iter()
            .map(|g| GroupOrUser::Group(&g.name, g.gid));
        let users = self.users.iter().map(|u| GroupOrUser::User(&u.name, u.id));

        for gu in groups.chain(users) {
            let id = gu.get_id();

            if let Some(dup) = reverse.get(&id) {
                match (gu, dup) {
                    (GroupOrUser::User(u1, _), GroupOrUser::User(u2, _)) => {
                        eprintln!("Users:{u1:?} and {u2:?} have the same id: {id}")
                    }
                    (GroupOrUser::User(u, _), GroupOrUser::Group(g, _)) => {
                        eprintln!("User {u:?} and Group {g:?} have the same id: {id}")
                    }
                    (GroupOrUser::Group(g, _), GroupOrUser::User(u, _)) => {
                        eprintln!("User {u:?} and Group {g:?} have the same id: {id}")
                    }
                    (GroupOrUser::Group(g1, _), GroupOrUser::Group(g2, _)) => {
                        eprintln!("Groups {g1:?} and {g2:?} have the same id: {id}")
                    }
                }
                return false;
            } else {
                reverse.insert(id, gu);
            }
        }
        true
    }
}

