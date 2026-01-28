mod handlers;
mod models;
mod utilities;

use axum::{
    Router, middleware,
    routing::{get, post},
};
use axum_server::tls_rustls::RustlsConfig;
use local_ip_address::local_ip;
use std::{error::Error, net::SocketAddr, sync::Arc};
use uuid::Uuid;
use clap::Parser;
use models::{AppState, Args};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let paths = utilities::get_canonical_paths(args.paths)?;
    let metadata = utilities::generate_metadata(&paths)?;
    let auth_token = Uuid::new_v4().to_string();

    let shared_state = Arc::new(AppState {
        metadata,
        password: args.password,
        auth_token,
    });

    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(3)
            .finish()
            .ok_or("could not create governor configuration")?
    );

    let app = Router::new()
        .route("/metadata", get(handlers::metadata))
        .route("/download/{uuid}", get(handlers::file))
        .layer(middleware::from_fn_with_state(shared_state.clone(), handlers::auth))
        .route("/", get(handlers::index))
        .route("/login", post(handlers::login).layer(GovernorLayer::new(governor_config)))
        .with_state(shared_state);

    let local_ip = local_ip()?;
    let addr = SocketAddr::new(local_ip, 8000);

    println!("Server running at https://{}", addr);
    println!("If you are using a firewall, you may need to expose the 8000 port.");
    println!("Press Ctrl+C to stop...");

    let (cert_pem, key_pem) = utilities::generate_cert(local_ip)?;
    let config = RustlsConfig::from_pem(cert_pem.into_bytes(), key_pem.into_bytes()).await?;

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}
