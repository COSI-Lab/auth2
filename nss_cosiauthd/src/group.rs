use futures::executor::block_on;
use libnss::interop::Response;
use tarpc::context;

use crate::RPC;

pub struct AuthdGroup {}

impl libnss::group::GroupHooks for AuthdGroup {
    fn get_all_entries() -> Response<Vec<libnss::group::Group>> {
        let mut cl = RPC.lock().unwrap();
        cl.with_client(
            |client| match block_on(client.get_all_groups(context::current())) {
                Ok(groups) => Response::Success(groups),
                Err(_) => Response::Unavail,
            },
        )
    }

    fn get_entry_by_gid(gid: libc::gid_t) -> Response<libnss::group::Group> {
        let mut cl = RPC.lock().unwrap();
        cl.with_client(
            |client| match block_on(client.get_group_by_gid(context::current(), gid)) {
                Ok(Some(group)) => Response::Success(group),
                Ok(None) => Response::NotFound,
                Err(_) => Response::Unavail,
            },
        )
    }

    fn get_entry_by_name(name: String) -> Response<libnss::group::Group> {
        let mut cl = RPC.lock().unwrap();
        cl.with_client(|client| {
            match block_on(client.get_group_by_name(context::current(), name)) {
                Ok(Some(group)) => Response::Success(group),
                Ok(None) => Response::NotFound,
                Err(_) => Response::Unavail,
            }
        })
    }
}
