use authd::types::GroupToNSS;
use futures::executor::block_on;
use libnss::interop::Response;
use tarpc::context;
use tracing::{info, warn};

use crate::RPC;

pub struct AuthdGroup {}

impl libnss::group::GroupHooks for AuthdGroup {
    fn get_all_entries() -> Response<Vec<libnss::group::Group>> {
        let mut cl = RPC.lock().unwrap();

        info!("get_all_groups");

        cl.with_client(
            |client| match block_on(client.get_all_groups(context::current())) {
                Ok(groups) => {
                    info!("get_all_groups success");
                    Response::Success(groups.to_nss())
                }
                Err(err) => {
                    warn!("get_all_groups unavail {}", err);
                    Response::Unavail
                }
            },
        )
    }

    fn get_entry_by_gid(gid: libc::gid_t) -> Response<libnss::group::Group> {
        let mut cl = RPC.lock().unwrap();

        info!("get_group_by_gid {}", gid);

        cl.with_client(
            |client| match block_on(client.get_group_by_gid(context::current(), gid)) {
                Ok(Some(group)) => {
                    info!("get_group_by_gid {} Success", gid);
                    Response::Success(group.to_nss())
                }
                Ok(None) => {
                    info!("get_group_by_gid {} NotFound", gid);
                    Response::NotFound
                }
                Err(err) => {
                    warn!("get_group_by_gid {} Unavail {}", gid, err);
                    Response::Unavail
                }
            },
        )
    }

    fn get_entry_by_name(name: String) -> Response<libnss::group::Group> {
        let mut cl = RPC.lock().unwrap();

        info!("get_group_by_name {}", name);

        cl.with_client(|client| {
            match block_on(client.get_group_by_name(context::current(), name.clone())) {
                Ok(Some(group)) => {
                    info!("get_group_by_name {} Success", name);
                    Response::Success(group.to_nss())
                }
                Ok(None) => {
                    info!("get_group_by_name {} NotFound", name);
                    Response::NotFound
                }
                Err(err) => {
                    warn!("get_group_by_name {} Unavail {}", name, err);
                    Response::Unavail
                }
            }
        })
    }
}
