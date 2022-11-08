use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::Value;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
}

pub trait GroupToNSS {
    type Target;
    fn to_nss(&self) -> Self::Target;
}

impl GroupToNSS for Group {
    type Target = libnss::group::Group;

    /// Creates a `libnss::group::Group` from a group config
    fn to_nss(&self) -> Self::Target {
        libnss::group::Group {
            name: self.name.clone(),
            passwd: "x".to_string(),
            gid: self.gid,
            members: self.members.clone(),
        }
    }
}

impl GroupToNSS for Vec<Group> {
    type Target = Vec<libnss::group::Group>;

    fn to_nss(&self) -> Self::Target {
        self.iter().map(|g| g.to_nss()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: u32,
    pub gecos: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub shells: Vec<Shell>,
}

pub trait UserToNSS {
    type Target;

    fn to_nss(&self, home_root: &str, shells_root: &str, shells: &Vec<Shell>) -> Self::Target;
}

impl UserToNSS for User {
    type Target = libnss::passwd::Passwd;

    /// Converts a `User` to a `libnss::passwd::Passwd`.
    ///
    /// Requires the path of where home folders are stored (such as "/mnt/home")
    /// and list of supported shells ie [Shell::Bash, Shell:Sh]
    ///
    /// Check `User::choose_shell` for documentation on `shells`
    fn to_nss(&self, home_root: &str, shells_root: &str, shells: &Vec<Shell>) -> Self::Target {
        libnss::passwd::Passwd {
            name: self.name.clone(),
            passwd: "x".to_string(),
            uid: self.id,
            gid: self.id,
            gecos: self.gecos_as_json(),
            dir: format!("{}/{}", home_root, self.name),
            shell: format!("{}/{}", shells_root, self.choose_shell(shells)),
        }
    }
}

impl User {
    /// Converts user's gecos struct into a JSON formatted string for use in programs
    fn gecos_as_json(&self) -> String {
        match &self.gecos {
            Some(map) => serde_json::to_string(&map).unwrap_or_default(),
            None => String::new(),
        }
    }

    /// Given a list of supported shells return the shell with the highest priority
    /// If for some
    fn choose_shell(&self, supported_shells: &Vec<Shell>) -> Shell {
        self.shells
            .iter()
            .find(|s| supported_shells.contains(s))
            .unwrap_or(&Shell::Bash)
            .clone()
    }
}

impl UserToNSS for Vec<User> {
    type Target = Vec<libnss::passwd::Passwd>;

    fn to_nss(&self, home_root: &str, shells_root: &str, shells: &Vec<Shell>) -> Self::Target {
        self.iter()
            .map(|user| user.to_nss(home_root, shells_root, shells))
            .collect()
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

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Shell::Sh => "sh",
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        };
        write!(f, "{}", s)
    }
}
