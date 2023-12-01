use futures::stream::StreamExt;
use structopt::StructOpt;

use ya_gcp::{
    bigtable::{self, admin::Rule},
    AuthFlow, ClientBuilder, ClientBuilderConfig, ServiceAccountAuth,
};

#[derive(Debug, StructOpt)]
struct Args {
    /// A path to the oauth service account key json file
    #[structopt(long)]
    service_account_key: Option<std::path::PathBuf>,

    #[structopt(long)]
    project_name: String,

    #[structopt(long)]
    instance_name: String,

    #[structopt(long)]
    table_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    let auth = if let Some(key) = args.service_account_key {
        AuthFlow::ServiceAccount(ServiceAccountAuth::Path(key))
    } else {
        AuthFlow::NoAuth
    };

    println!("Creating clients");

    let config = ClientBuilderConfig::new().auth_flow(auth);
    let builder = ClientBuilder::new(config).await?;

    let mut bigtable_admin_config = bigtable::admin::BigtableTableAdminConfig::default();
    let mut bigtable_config = bigtable::BigtableConfig::default();

    if let Ok(host) = std::env::var("BIGTABLE_EMULATOR_HOST") {
        bigtable_admin_config.endpoint = host.clone();
        bigtable_config.endpoint = host;
    };

    let mut admin = builder
        .build_bigtable_admin_client(
            bigtable_admin_config,
            &args.project_name,
            &args.instance_name,
        )
        .await?;

    println!("Creating table `{}`", &args.table_name);

    match admin
        .create_table(
            &args.table_name,
            [("family-name".to_owned(), Rule::MaxNumVersions(10))],
        )
        .await
    {
        Ok(table) => println!("created table {:?}", table),
        Err(e) => {
            if e.code() == tonic::Code::AlreadyExists {
                println!("table already exists");
            } else {
                println!("unexpected error");
                Err(e)?;
            }
        }
    }

    println!("Reading tables");

    let tables: Vec<_> = admin.list_tables().await?.collect().await;

    println!("got tables {:?}", tables);

    let mut client = builder
        .build_bigtable_client(bigtable_config, &args.project_name, &args.instance_name)
        .await?;

    println!("setting data");

    client
        .set_row_data(
            &args.table_name,
            "family-name".to_string(),
            "row00",
            [("col1", "value"), ("col2", "value")],
        )
        .await?;

    println!("set data done");
    println!(
        "all data: {:?}",
        client
            .read_row_range(&args.table_name, .., Some(100))
            .collect::<Vec<_>>()
            .await
    );

    Ok(())
}
