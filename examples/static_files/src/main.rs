//! An example of serving static files with Gotham.

extern crate gotham;

use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use gotham::staticfile::StaticFileHandler;
use std::path::PathBuf;

pub fn main() {
    let path = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| panic!("Need to pass an arg which is path to serve")),
    );
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);

    let router =
        build_simple_router(|route| route.get("/*").to_filesystem(StaticFileHandler::new(path)));

    gotham::start(addr, router)
}
