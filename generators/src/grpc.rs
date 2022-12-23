use anyhow::{Context, Error};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

/// An application used to fetch gRPC schemas and generate rust code for `ya-gcp`
///
/// This is expected to be run manually, with the generated code moved into the `ya-gcp`
/// crate as needed.
#[derive(StructOpt)]
struct Args {
    /// A path to a directory containing the protobuf schemas for Google service APIs. If not
    /// provided, the schemas will be fetched from the [googleapis
    /// repo](https://github.com/googleapis/googleapis)
    #[structopt(long)]
    google_protos: Option<PathBuf>,

    /// A path to the directory where the generated files will be written
    #[structopt(long)]
    output_dir: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    // if fetching, keep around the tempdir so that it gets deleted after everything finishes
    let (google_protos, _temp_dir) = match args.google_protos {
        Some(existing) => (existing, None),
        None => {
            let tempdir = tempfile::tempdir().context("failed to create temporary directory")?;
            (
                fetch_google_apis(&tempdir).context("failed to fetch google protobuf schemas")?,
                Some(tempdir),
            )
        }
    };

    std::fs::create_dir_all(&args.output_dir).context("failed to create output directory")?;

    println!("generating interfaces in {}", args.output_dir.display());

    let mut prost_config = prost_build::Config::new();
    // use Bytes instead of Vec<u8> when possible in order to reduce copies when receiving data off
    // the wire
    prost_config.bytes(&["."]);

    // The bigtable docs have doc comments that trigger test failures.
    // (TODO: in newer versions of prost-build, the `format` option might be enough for this)
    prost_config.disable_comments(&[
        "bigtable.v2.RowFilter.Interleave.filters",
        "bigtable.v2.RowFilter.sink",
        "iam.v1.Policy",
        "iam.v1.AuditConfig",
        "iam.v1.AuditLogConfig",
        "type.Expr",
    ]);

    // the attributes map tend to have a small number of string keys, which are faster to access
    // using a btree than a hashmap. See the crate's benchmarks
    prost_config.btree_map(&["PubsubMessage.attributes"]);

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .format(true)
        .out_dir(&args.output_dir)
        .compile_with_config(
            prost_config,
            [
                "google/pubsub/v1/pubsub.proto",
                "google/bigtable/v2/bigtable.proto",
                "google/bigtable/admin/v2/bigtable_table_admin.proto",
            ]
            .iter()
            .map(|src| google_protos.join(src))
            .collect::<Vec<_>>()
            .as_ref(),
            &[google_protos],
        )
        .context("failed to generate rust sources")?;

    Ok(())
}

/// Get the google apis from their public repository and write them to the given directory.
///
/// returns the subpath at which the protobuf directories are at top-level
fn fetch_google_apis(destination_dir: impl AsRef<Path>) -> Result<PathBuf, Error> {
    const TAR_NAME: &str = "googleapis-master";
    const URL: &str = "https://github.com/googleapis/googleapis/archive/master.tar.gz";

    println!("fetching schemas from {}...", URL);
    let tarball = reqwest::blocking::get(URL)
        .context("failed to download api tarball")?
        .bytes()?;

    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&tarball[..]));

    println!("unpacking schemas...");
    archive.unpack(&destination_dir)?;

    // the tar is unpacked in the destination, but we need one layer down inside the tar
    Ok(destination_dir.as_ref().join(TAR_NAME))
}
