#[cfg(all(feature = "bigtable", feature = "emulators"))]
mod bigtable_client_tests {
    use futures::TryStreamExt;
    use hyper::body::Bytes;
    use ya_gcp::bigtable::{
        self,
        admin::Rule,
        api,
        api::bigtable::v2,
        emulator::{Emulator, EmulatorClient},
        mutation::{
            self,
            v2::{row_filter, RowFilter},
        },
        CheckAndMutateRowRequest, ReadRowsRequest,
    };

    #[tokio::test]
    async fn create_table() {
        let table_name = "test-table";
        let emulator = Emulator::new()
            .project("test-project")
            .instance("test-instance")
            .await
            .unwrap();
        let config = bigtable::admin::BigtableTableAdminConfig::new().endpoint(emulator.endpoint());
        let mut admin = emulator
            .builder()
            .build_bigtable_admin_client(config, emulator.project(), emulator.instance())
            .await
            .unwrap();

        admin
            .create_table(table_name, [("column".into(), Rule::MaxNumVersions(1))])
            .await
            .unwrap();

        let tables: Vec<_> = admin
            .list_tables()
            .await
            .unwrap()
            .try_collect()
            .await
            .unwrap();

        assert_eq!(tables.len(), 1);
        assert_eq!(
            tables[0].name,
            "projects/test-project/instances/test-instance/tables/test-table"
        );
    }

    async fn default_client(table_name: &str) -> (EmulatorClient, bigtable::BigtableClient) {
        let emulator = Emulator::new()
            .project("test-project")
            .instance("test-instance")
            .await
            .unwrap();
        emulator
            .create_table(table_name, ["fam1", "fam2"])
            .await
            .unwrap();

        let config = bigtable::BigtableConfig::new().endpoint(emulator.endpoint());
        let client = emulator
            .builder()
            .build_bigtable_client(config, emulator.project(), emulator.instance())
            .await
            .unwrap();
        (emulator, client)
    }

    #[tokio::test]
    async fn set_and_read_row() {
        let table_name = "test-table";
        let (_emulator, mut client) = default_client(table_name).await;

        client
            .set_row_data(
                table_name,
                "fam1".into(),
                "row1-key",
                [("col1", "data1"), ("col2", "data2")],
            )
            .await
            .unwrap();

        let row = client
            .read_one_row(table_name, "row1-key")
            .await
            .unwrap()
            .unwrap();
        assert_eq!("row1-key", row.key);
        assert_eq!(1, row.families.len());
        assert_eq!("fam1", row.families[0].name);
        assert_eq!(2, row.families[0].columns.len());
        assert_eq!("col1", row.families[0].columns[0].qualifier);
        assert_eq!("data1", row.families[0].columns[0].cells[0].value);
        assert_eq!("col2", row.families[0].columns[1].qualifier);
        assert_eq!("data2", row.families[0].columns[1].cells[0].value);

        let row_range: Vec<_> = client
            .read_row_range(
                table_name,
                Bytes::from("row1-key")..=Bytes::from("row1-key"),
                None,
            )
            .try_collect()
            .await
            .unwrap();
        assert_eq!(row_range, vec![row]);
    }

    #[tokio::test]
    async fn cell_versions() {
        let table_name = "test-table";
        let (emulator, mut client) = default_client(table_name).await;
        client
            .set_row_data_with_timestamp(
                table_name,
                "fam1".into(),
                // The bigtable emulator enforces millisecond granularity
                6000,
                "row1-key",
                [("col1", "data1")],
            )
            .await
            .unwrap();
        client
            .set_row_data_with_timestamp(
                table_name,
                "fam1".into(),
                7000,
                "row1-key",
                [("col1", "data2")],
            )
            .await
            .unwrap();
        let row = client
            .read_one_row(table_name, "row1-key")
            .await
            .unwrap()
            .unwrap();
        // read_one_row only returns the latest version
        assert_eq!(1, row.families[0].columns[0].cells.len());
        assert_eq!("data2", row.families[0].columns[0].cells[0].value);
        assert_eq!(
            vec!["data2".as_bytes()],
            row.most_recent_cells().map(|c| c.value).collect::<Vec<_>>()
        );

        let req = {
            let mut r = ReadRowsRequest::default();
            r.table_name = format!(
                "projects/{}/instances/{}/tables/{table_name}",
                emulator.project(),
                emulator.instance()
            );
            r.rows = Some(api::bigtable::v2::RowSet::default().with_key("row1-key"));
            r
        };
        let rows: Vec<_> = client.read_rows(req).try_collect().await.unwrap();
        assert_eq!(1, rows.len());
        let row = &rows[0];
        assert_eq!(2, row.families[0].columns[0].cells.len());
        assert_eq!(
            vec!["data2".as_bytes()],
            row.most_recent_cells().map(|c| c.value).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn conditional_mutate() {
        let table_name = "test-table";
        let (_emulator, mut client) = default_client(table_name).await;

        client
            .set_row_data(
                table_name,
                "fam1".into(),
                "row1-key",
                [("col1", "data1"), ("col2", "data2")],
            )
            .await
            .unwrap();

        let mut mutation = v2::Mutation::default();
        mutation.mutation = Some(
            mutation::SetCell::new(
                "fam1".to_owned(),
                Bytes::from_static(b"col1"),
                Bytes::from_static(b"new"),
            )
            .with_timestamp(-1)
            .into(),
        );

        let mut req = CheckAndMutateRowRequest::default();
        let mut filter = RowFilter::default();
        filter.filter = Some(row_filter::Filter::ValueRegexFilter(Bytes::from_static(
            b"data3",
        )));
        req.table_name = client.fully_qualified_table_name(table_name);
        req.row_key = Bytes::from_static(b"row1-key");
        req.predicate_filter = Some(filter.into());
        req.true_mutations = vec![mutation];

        // This filter shouldn't match.
        assert!(!client.check_and_mutate_row(req.clone()).await.unwrap());
        let row = client
            .read_one_row(table_name, "row1-key")
            .await
            .unwrap()
            .unwrap();
        assert_eq!("data1", row.families[0].columns[0].cells[0].value);
        assert_eq!("data2", row.families[0].columns[1].cells[0].value);

        req.predicate_filter.as_mut().unwrap().filter = Some(row_filter::Filter::ValueRegexFilter(
            Bytes::from_static(b"data2"),
        ));
        // This filter should match.
        assert!(client.check_and_mutate_row(req.clone()).await.unwrap());
        let row = client
            .read_one_row(table_name, "row1-key")
            .await
            .unwrap()
            .unwrap();
        assert_eq!("new", row.families[0].columns[0].cells[0].value);
        assert_eq!("data2", row.families[0].columns[1].cells[0].value);
    }
}
