use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(long)]
    pub auth0_domain: String,

    #[clap(long)]
    pub auth0_client_id: String,

    #[clap(long)]
    pub audience: String,

    #[clap(long)]
    pub auth0_client_secret: String,

    #[clap(long)]
    pub auth0_scopes: String,
}