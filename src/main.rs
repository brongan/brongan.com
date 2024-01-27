#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    brongan_com::server::run().await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
