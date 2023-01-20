use futures::executor::block_on;
use libnss::interop::Response;
use tarpc::context;
use tracing::{error, info, warn};

use crate::{CFG, RPC};

pub struct AuthdPasswd {}

impl libnss::passwd::PasswdHooks for AuthdPasswd {
    fn get_all_entries() -> Response<Vec<libnss::passwd::Passwd>> {
        let mut cl = RPC.lock().unwrap();

        info!("get_all_passwd");

        let cfg = match &*CFG {
            Ok(cfg) => cfg,
            Err(_) => {
                error!("get_all_passwd missing config");
                return Response::Unavail;
            }
        };

        cl.with_client(
            |client| match block_on(client.get_all_passwd(context::current())) {
                Ok(passwds) => {
                    info!("get_all_passwd Success");
                    Response::Success(passwds.to_nss(&cfg.home_root, &cfg.shells_root, &cfg.shells))
                }
                Err(err) => {
                    warn!("get_all_passwd Unavail {}", err);
                    Response::Unavail
                }
            },
        )
    }

    fn get_entry_by_uid(uid: libc::uid_t) -> Response<libnss::passwd::Passwd> {
        let mut cl = RPC.lock().unwrap();

        info!("get_passwd_by_uid {}", uid);

        let cfg = match &*CFG {
            Ok(cfg) => cfg,
            Err(_) => {
                error!("get_passwd_by_uid {} missing config", uid);
                return Response::Unavail;
            }
        };

        cl.with_client(
            |client| match block_on(client.get_passwd_by_uid(context::current(), uid)) {
                Ok(Some(passwd)) => {
                    info!("get_passwd_by_uid {} Success", uid);
                    Response::Success(passwd.to_nss(&cfg.home_root, &cfg.shells_root, &cfg.shells))
                }
                Ok(None) => {
                    info!("get_passwd_by_uid {} NotFound", uid);
                    Response::NotFound
                }
                Err(err) => {
                    warn!("get_passwd_by_uid {} Unavail {}", uid, err);
                    Response::Unavail
                }
            },
        )
    }

    fn get_entry_by_name(name: String) -> Response<libnss::passwd::Passwd> {
        let mut cl = RPC.lock().unwrap();

        info!("get_passwd_by_name {}", name);

        let cfg = match &*CFG {
            Ok(cfg) => cfg,
            Err(_) => {
                error!("get_passwd_by_name {} missing config", name);
                return Response::Unavail;
            }
        };

        cl.with_client(|client| {
            match block_on(client.get_passwd_by_name(context::current(), name.clone())) {
                Ok(Some(passwd)) => {
                    info!("get_passwd_by_name {} Success", name);
                    Response::Success(passwd.to_nss(&cfg.home_root, &cfg.shells_root, &cfg.shells))
                }
                Ok(None) => {
                    info!("get_passwd_by_name {} NotFound", name);
                    Response::NotFound
                }
                Err(err) => {
                    warn!("get_passwd_by_name {} Unavail {}", name, err);
                    Response::Unavail
                }
            }
        })
    }
}
