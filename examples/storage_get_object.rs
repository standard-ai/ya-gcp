//! An example of downloading an object from GCS.

use structopt::StructOpt;

use ya_gcp::{AuthFlow, ClientBuilder, ClientBuilderConfig, ServiceAccountAuth};

#[derive(Debug, StructOpt)]
struct Args {
    /// A path to the oauth service account key json file.
    ///
    /// If this is not provided, `user_secrets` must be.
    #[structopt(long)]
    service_account_key: Option<std::path::PathBuf>,

    /// Path to oauth user secrets, like those created by
    /// `gcloud auth application-default login`
    ///
    /// If this is not provided, `service_account_key` must be.
    #[structopt(long)]
    user_secrets: Option<std::path::PathBuf>,

    /// The name of the bucket containing the desired object
    #[structopt(long)]
    bucket_name: String,

    /// The name of the object to get
    #[structopt(long)]
    object_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    let auth = if let Some(path) = args.service_account_key {
        AuthFlow::ServiceAccount(ServiceAccountAuth::Path(path))
    } else if let Some(path) = args.user_secrets {
        AuthFlow::UserAccount(path)
    } else {
        panic!("at least one of service-account-key or user-secrets must be specified");
    };

    let builder = ClientBuilder::new(ClientBuilderConfig::new().auth_flow(auth)).await?;

    let client = builder.build_storage_client();

    println!(
        "{:?}",
        client
            .get_object(args.bucket_name, args.object_name)
            .await?
    );

    Ok(())
}
