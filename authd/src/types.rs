use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub groups: HashMap<String, Group>,
    pub users: HashMap<String, User>,
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
            .map(|(n, g)| GroupOrUser::Group(n, g.gid));
        let users = self.users.iter().map(|(n, u)| GroupOrUser::User(n, u.id));

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub gid: u32,
    pub members: Vec<String>,
}

impl Group {
    /// Creates a `libnss::group::Group` from a group config
    pub fn to_nss(&self, name: &str) -> libnss::group::Group {
        libnss::group::Group {
            name: name.to_string(),
            passwd: "x".to_string(),
            gid: self.gid,
            members: self.members.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub gecos: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub shells: Vec<Shell>,
}

impl User {
    /// Converts a `User` to a `libnss::passwd::Passwd`. 
    ///
    /// Requires the path of where home folders are stored (such as "/mnt/home")
    /// and list of supported shells ie [Shell::Bash, Shell:Sh]
    ///
    /// Check `User::choose_shell` for documentation on `shells`
    pub fn to_nss(
        &self,
        name: &str,
        home_root: &str,
        shells: &Option<Vec<Shell>>,
    ) -> libnss::passwd::Passwd {
        libnss::passwd::Passwd {
            name: name.to_string(),
            passwd: "x".to_string(),
            uid: self.id,
            gid: self.id,
            gecos: self.gecos_as_json(),
            dir: format!("{}/{}", home_root, name),
            shell: self.choose_shell(&shells).to_string(),
        }
    }

    /// Converts user's gecos struct into a JSON formatted string for use in programs
    fn gecos_as_json(&self) -> String {
        match &self.gecos {
            Some(map) => serde_json::to_string(&map).unwrap_or(String::new()),
            None => String::new(),
        }
    }

    /// Given a list of supported shells return the shell with the highest priority
    ///
    /// If no supported shells are provided returns the shell with highest priority, or if no
    /// priorities are provided returns Shell::Bash
    ///
    /// If no priorities are not assigned returns Shell::Bash if it's supported.
    /// If not it defaults to the first shell that is supported
    fn choose_shell(&self, supported_shells: &Option<Vec<Shell>>) -> Shell {
        match supported_shells {
            Some(supported_shells) => *self
                .shells
                .iter()
                .find(|s| supported_shells.contains(s))
                .unwrap_or(&Shell::Bash),
            None => *self.shells.get(0).unwrap_or(&Shell::Bash),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Sh,
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    fn to_string(&self) -> String {
        match self {
            Shell::Sh => "sh",
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
        .to_string()
    }
}
