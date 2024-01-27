use brongan_com::server;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    server::run().await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
