use anyhow::{Context, Error};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
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

    #[structopt(long, case_insensitive = true, default_value = "client")]
    mode: Mode,

    #[structopt(long)]
    include_serde_impls: bool,

    /// A path to the directory where the generated files will be written
    #[structopt(long)]
    output_dir: PathBuf,
}

enum Mode {
    Client,
    Server,
}

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(Self::Client),
            "server" => Ok(Self::Server),
            _ => Err("possible values are 'client' or 'server'"),
        }
    }
}

impl Mode {
    fn should_build_client(&self) -> bool {
        matches!(self, Self::Client)
    }

    fn should_build_server(&self) -> bool {
        matches!(self, Self::Server)
    }
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

    let descriptor_path = args.output_dir.clone().join("proto_descriptor.bin");

    std::fs::create_dir_all(&args.output_dir).context("failed to create output directory")?;

    println!("generating interfaces in {}", args.output_dir.display());

    let mut prost_config = prost_build::Config::new();
    prost_config.format(true);
    // use Bytes instead of Vec<u8> when possible in order to reduce copies when receiving data off
    // the wire
    prost_config.bytes(&["."]);

    if args.include_serde_impls {
        prost_config
            .file_descriptor_set_path(&descriptor_path)
            .compile_well_known_types()
            .extern_path(".google.protobuf", "::pbjson_types");
    }

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
        .build_client(args.mode.should_build_client())
        .build_server(args.mode.should_build_server())
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

    if args.include_serde_impls {
        let descriptor_set = std::fs::read(descriptor_path)?;
        pbjson_build::Builder::new()
            .out_dir(&args.output_dir)
            .register_descriptors(&descriptor_set)?
            .build(&[".google.bigtable.v2", ".google.rpc"])?;
    }

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
