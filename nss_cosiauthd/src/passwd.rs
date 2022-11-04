use futures::executor::block_on;
use libnss::interop::Response;
use tarpc::context;

use crate::RPC;

pub struct AuthdPasswd {}

impl libnss::passwd::PasswdHooks for AuthdPasswd {
    fn get_all_entries() -> Response<Vec<libnss::passwd::Passwd>> {
        let mut cl = RPC.lock().unwrap();
        cl.with_client(
            |client| match block_on(client.get_all_passwd(context::current())) {
                Ok(passwds) => Response::Success(passwds),
                Err(_) => Response::Unavail,
            },
        )
    }

    fn get_entry_by_uid(uid: libc::uid_t) -> Response<libnss::passwd::Passwd> {
        let mut cl = RPC.lock().unwrap();
        cl.with_client(
            |client| match block_on(client.get_passwd_by_uid(context::current(), uid)) {
                Ok(Some(passwd)) => Response::Success(passwd),
                Ok(None) => Response::NotFound,
                Err(_) => Response::Unavail,
            },
        )
    }

    fn get_entry_by_name(name: String) -> Response<libnss::passwd::Passwd> {
        let mut cl = RPC.lock().unwrap();
        cl.with_client(|client| {
            match block_on(client.get_passwd_by_name(context::current(), name)) {
                Ok(Some(passwd)) => Response::Success(passwd),
                Ok(None) => Response::NotFound,
                Err(_) => Response::Unavail,
            }
        })
    }
}
