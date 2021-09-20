//! An example of downloading an object from GCS.

use structopt::StructOpt;

use ya_gcp::{AuthFlow, ClientBuilder, ClientBuilderConfig, ServiceAccountAuth};

#[derive(Debug, StructOpt)]
struct Args {
    /// A path to the oauth service account key json file
    #[structopt(long)]
    service_account_key: std::path::PathBuf,

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

    let builder = ClientBuilder::new(ClientBuilderConfig::new().auth_flow(
        AuthFlow::ServiceAccount(ServiceAccountAuth::Path(args.service_account_key)),
    ))
    .await?;

    let client = builder.build_storage_client();

    println!(
        "{:?}",
        client
            .get_object(args.bucket_name, args.object_name)
            .await?
    );

    Ok(())
}
