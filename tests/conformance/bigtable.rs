use std::{fs::File, path::PathBuf};

use futures::StreamExt;
use serde::Deserialize;
use ya_gcp::{
    bigtable::{BigtableClient, BigtableConfig, ReadRowsError, Row},
    AuthFlow, ClientBuilder, ClientBuilderConfig,
};

use crate::bigtable_stub::{api::bigtable::v2 as bigtable, StubBigtableServer};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestFile {
    read_rows_tests: Vec<ReadRowsTest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadRowsTest {
    description: String,
    chunks: Vec<bigtable::read_rows_response::CellChunk>,
    #[serde(default = "Vec::new")]
    results: Vec<ReadRowsResult>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadRowsResult {
    // Broadly speaking we want Go-like semantics for the values here,
    // which is to say that if the value is missing from the JSON blob
    // then we should assume it's the empty string.
    #[serde(default = "String::new")]
    family_name: String,
    #[serde(default = "String::new")]
    qualifier: String,
    #[serde(default = "String::new")]
    value: String,
    // If this is present we parse it as an i64, and if it's missing we
    // default to 0.
    timestamp_micros: Option<String>,
    // If this is missing, we default to false.
    error: Option<bool>,
}

#[tokio::test]
async fn bigtable_conformance_tests() {
    let test_file = parse_test_data();

    for test_case in test_file.read_rows_tests {
        // TODO: consider making this configurable via env var
        let addr = "127.0.0.1:8081";
        let _server_handle =
            StubBigtableServer::run_with_chunks(addr.parse().unwrap(), &test_case.chunks).await;

        let mut client = build_client(format!("http://{}", addr)).await;

        // The actual request doesn't matter since we're handling stubbed responses
        let rows = client
            .read_rows(ya_gcp::bigtable::ReadRowsRequest::default())
            .collect::<Vec<_>>()
            .await;

        check_results(test_case, rows);
    }
}

fn parse_test_data() -> TestFile {
    let test_data_file =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/conformance/data/readrows.json");
    let test_data_json =
        File::open(test_data_file.as_path()).expect("Could not open readrows.json");
    serde_json::from_reader(test_data_json).expect("Could not parse readrows.json")
}

async fn build_client(endpoint: String) -> BigtableClient {
    let client_cfg = ClientBuilderConfig::new().auth_flow(AuthFlow::NoAuth);
    let bt_cfg = BigtableConfig::new().endpoint(endpoint);
    ClientBuilder::new(client_cfg)
        .await
        .expect("Failed to create ClientBuilder")
        // Project & instance don't matter as we're working with stubbed responses
        .build_bigtable_client(bt_cfg, "some-project", "some-instance")
        .await
        .expect("Failed to build BigtableClient")
}

fn check_results(mut test_case: ReadRowsTest, mut rows: Vec<Result<Row, ReadRowsError>>) {
    let should_error = test_case
        .results
        .last()
        .map(|r| r.error.is_some())
        .unwrap_or(false);
    if should_error {
        assert!(
            rows.last().map(|r| r.is_err()).unwrap_or(false),
            "Failure: {}, expected an error but didn't get one",
            test_case.description,
        );
        rows.pop();
        test_case.results.pop();
    }

    let rows = rows.into_iter().collect::<Result<Vec<_>, _>>().expect(
        format!(
            "Failure: {}, unexpected error when reading rows",
            test_case.description,
        )
        .as_str(),
    );

    // The conformance test JSON has its results formatted as a list of
    // cells, with their family name and column qualifier, sorted by
    // family name.
    struct ReceivedCell {
        family_name: String,
        column_qualifier: prost::bytes::Bytes,
        cell: ya_gcp::bigtable::Cell,
    }

    let items: Vec<ReceivedCell> = rows
        .into_iter()
        .flat_map(|r| {
            let mut families = r.families.clone();
            families.sort_by(|f1, f2| f1.name.cmp(&f2.name));

            families
                .into_iter()
                .flat_map(|f| {
                    let f_name = f.name.clone();
                    f.columns.into_iter().map(move |c| (f_name.clone(), c))
                })
                .flat_map(|(f_name, col)| {
                    let col_qualifier = col.qualifier.clone();
                    col.cells.into_iter().map(move |c| ReceivedCell {
                        family_name: f_name.clone(),
                        column_qualifier: col_qualifier.clone(),
                        cell: c,
                    })
                })
        })
        .collect();

    assert_eq!(
        items.len(),
        test_case.results.len(),
        "Failure: {}, wrong number of cells in response",
        test_case.description,
    );

    for (got, want) in items.into_iter().zip(test_case.results.iter()) {
        let value_got = String::from_utf8(got.cell.value.as_ref().to_owned())
            .expect("Values should be valid UTF-8");
        let value_want = want.value.clone();
        assert_eq!(
            value_got, value_want,
            "Failure: {}, values differed",
            test_case.description
        );

        let family_name_got = got.family_name;
        let family_name_want = want.family_name.clone();
        assert_eq!(
            family_name_want, family_name_got,
            "Failure: {}, family names differed",
            test_case.description
        );

        let ts_micros_got = got.cell.timestamp_micros;
        let ts_micros_want: i64 = want
            .timestamp_micros
            .as_ref()
            .and_then(|t| t.parse().ok())
            .unwrap_or_default();
        assert_eq!(
            ts_micros_got, ts_micros_want,
            "Failure: {}, timestamps differed",
            test_case.description
        );

        let column_got = String::from_utf8(got.column_qualifier.as_ref().to_owned())
            .expect("Column name should be valid UTF-8");
        let column_want = want.qualifier.clone();
        assert_eq!(
            column_got, column_want,
            "Failure: {}, column qualifiers differed",
            test_case.description
        );
    }
}
