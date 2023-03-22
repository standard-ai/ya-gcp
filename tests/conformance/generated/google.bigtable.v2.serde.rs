impl serde::Serialize for Cell {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.timestamp_micros != 0 {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        if !self.labels.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Cell", len)?;
        if self.timestamp_micros != 0 {
            struct_ser.serialize_field("timestampMicros", ToString::to_string(&self.timestamp_micros).as_str())?;
        }
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", pbjson::private::base64::encode(&self.value).as_str())?;
        }
        if !self.labels.is_empty() {
            struct_ser.serialize_field("labels", &self.labels)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Cell {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timestamp_micros",
            "timestampMicros",
            "value",
            "labels",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TimestampMicros,
            Value,
            Labels,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "timestampMicros" | "timestamp_micros" => Ok(GeneratedField::TimestampMicros),
                            "value" => Ok(GeneratedField::Value),
                            "labels" => Ok(GeneratedField::Labels),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Cell;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Cell")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Cell, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timestamp_micros__ = None;
                let mut value__ = None;
                let mut labels__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TimestampMicros => {
                            if timestamp_micros__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestampMicros"));
                            }
                            timestamp_micros__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Labels => {
                            if labels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("labels"));
                            }
                            labels__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Cell {
                    timestamp_micros: timestamp_micros__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                    labels: labels__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Cell", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CheckAndMutateRowRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        if !self.row_key.is_empty() {
            len += 1;
        }
        if self.predicate_filter.is_some() {
            len += 1;
        }
        if !self.true_mutations.is_empty() {
            len += 1;
        }
        if !self.false_mutations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.CheckAndMutateRowRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        if !self.row_key.is_empty() {
            struct_ser.serialize_field("rowKey", pbjson::private::base64::encode(&self.row_key).as_str())?;
        }
        if let Some(v) = self.predicate_filter.as_ref() {
            struct_ser.serialize_field("predicateFilter", v)?;
        }
        if !self.true_mutations.is_empty() {
            struct_ser.serialize_field("trueMutations", &self.true_mutations)?;
        }
        if !self.false_mutations.is_empty() {
            struct_ser.serialize_field("falseMutations", &self.false_mutations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CheckAndMutateRowRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
            "row_key",
            "rowKey",
            "predicate_filter",
            "predicateFilter",
            "true_mutations",
            "trueMutations",
            "false_mutations",
            "falseMutations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
            RowKey,
            PredicateFilter,
            TrueMutations,
            FalseMutations,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            "rowKey" | "row_key" => Ok(GeneratedField::RowKey),
                            "predicateFilter" | "predicate_filter" => Ok(GeneratedField::PredicateFilter),
                            "trueMutations" | "true_mutations" => Ok(GeneratedField::TrueMutations),
                            "falseMutations" | "false_mutations" => Ok(GeneratedField::FalseMutations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CheckAndMutateRowRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.CheckAndMutateRowRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CheckAndMutateRowRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                let mut row_key__ = None;
                let mut predicate_filter__ = None;
                let mut true_mutations__ = None;
                let mut false_mutations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::RowKey => {
                            if row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKey"));
                            }
                            row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PredicateFilter => {
                            if predicate_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("predicateFilter"));
                            }
                            predicate_filter__ = map.next_value()?;
                        }
                        GeneratedField::TrueMutations => {
                            if true_mutations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trueMutations"));
                            }
                            true_mutations__ = Some(map.next_value()?);
                        }
                        GeneratedField::FalseMutations => {
                            if false_mutations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("falseMutations"));
                            }
                            false_mutations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CheckAndMutateRowRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                    row_key: row_key__.unwrap_or_default(),
                    predicate_filter: predicate_filter__,
                    true_mutations: true_mutations__.unwrap_or_default(),
                    false_mutations: false_mutations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.CheckAndMutateRowRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CheckAndMutateRowResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.predicate_matched {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.CheckAndMutateRowResponse", len)?;
        if self.predicate_matched {
            struct_ser.serialize_field("predicateMatched", &self.predicate_matched)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CheckAndMutateRowResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "predicate_matched",
            "predicateMatched",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PredicateMatched,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "predicateMatched" | "predicate_matched" => Ok(GeneratedField::PredicateMatched),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CheckAndMutateRowResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.CheckAndMutateRowResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CheckAndMutateRowResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut predicate_matched__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PredicateMatched => {
                            if predicate_matched__.is_some() {
                                return Err(serde::de::Error::duplicate_field("predicateMatched"));
                            }
                            predicate_matched__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CheckAndMutateRowResponse {
                    predicate_matched: predicate_matched__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.CheckAndMutateRowResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Column {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.qualifier.is_empty() {
            len += 1;
        }
        if !self.cells.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Column", len)?;
        if !self.qualifier.is_empty() {
            struct_ser.serialize_field("qualifier", pbjson::private::base64::encode(&self.qualifier).as_str())?;
        }
        if !self.cells.is_empty() {
            struct_ser.serialize_field("cells", &self.cells)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Column {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "qualifier",
            "cells",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Qualifier,
            Cells,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "qualifier" => Ok(GeneratedField::Qualifier),
                            "cells" => Ok(GeneratedField::Cells),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Column;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Column")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Column, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut qualifier__ = None;
                let mut cells__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Qualifier => {
                            if qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("qualifier"));
                            }
                            qualifier__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Cells => {
                            if cells__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cells"));
                            }
                            cells__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Column {
                    qualifier: qualifier__.unwrap_or_default(),
                    cells: cells__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Column", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ColumnRange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.family_name.is_empty() {
            len += 1;
        }
        if self.start_qualifier.is_some() {
            len += 1;
        }
        if self.end_qualifier.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ColumnRange", len)?;
        if !self.family_name.is_empty() {
            struct_ser.serialize_field("familyName", &self.family_name)?;
        }
        if let Some(v) = self.start_qualifier.as_ref() {
            match v {
                column_range::StartQualifier::StartQualifierClosed(v) => {
                    struct_ser.serialize_field("startQualifierClosed", pbjson::private::base64::encode(&v).as_str())?;
                }
                column_range::StartQualifier::StartQualifierOpen(v) => {
                    struct_ser.serialize_field("startQualifierOpen", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        if let Some(v) = self.end_qualifier.as_ref() {
            match v {
                column_range::EndQualifier::EndQualifierClosed(v) => {
                    struct_ser.serialize_field("endQualifierClosed", pbjson::private::base64::encode(&v).as_str())?;
                }
                column_range::EndQualifier::EndQualifierOpen(v) => {
                    struct_ser.serialize_field("endQualifierOpen", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ColumnRange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "family_name",
            "familyName",
            "start_qualifier_closed",
            "startQualifierClosed",
            "start_qualifier_open",
            "startQualifierOpen",
            "end_qualifier_closed",
            "endQualifierClosed",
            "end_qualifier_open",
            "endQualifierOpen",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FamilyName,
            StartQualifierClosed,
            StartQualifierOpen,
            EndQualifierClosed,
            EndQualifierOpen,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "familyName" | "family_name" => Ok(GeneratedField::FamilyName),
                            "startQualifierClosed" | "start_qualifier_closed" => Ok(GeneratedField::StartQualifierClosed),
                            "startQualifierOpen" | "start_qualifier_open" => Ok(GeneratedField::StartQualifierOpen),
                            "endQualifierClosed" | "end_qualifier_closed" => Ok(GeneratedField::EndQualifierClosed),
                            "endQualifierOpen" | "end_qualifier_open" => Ok(GeneratedField::EndQualifierOpen),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ColumnRange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ColumnRange")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ColumnRange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut family_name__ = None;
                let mut start_qualifier__ = None;
                let mut end_qualifier__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FamilyName => {
                            if family_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("familyName"));
                            }
                            family_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::StartQualifierClosed => {
                            if start_qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startQualifierClosed"));
                            }
                            start_qualifier__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| column_range::StartQualifier::StartQualifierClosed(x.0));
                        }
                        GeneratedField::StartQualifierOpen => {
                            if start_qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startQualifierOpen"));
                            }
                            start_qualifier__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| column_range::StartQualifier::StartQualifierOpen(x.0));
                        }
                        GeneratedField::EndQualifierClosed => {
                            if end_qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endQualifierClosed"));
                            }
                            end_qualifier__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| column_range::EndQualifier::EndQualifierClosed(x.0));
                        }
                        GeneratedField::EndQualifierOpen => {
                            if end_qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endQualifierOpen"));
                            }
                            end_qualifier__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| column_range::EndQualifier::EndQualifierOpen(x.0));
                        }
                    }
                }
                Ok(ColumnRange {
                    family_name: family_name__.unwrap_or_default(),
                    start_qualifier: start_qualifier__,
                    end_qualifier: end_qualifier__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ColumnRange", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Family {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.columns.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Family", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.columns.is_empty() {
            struct_ser.serialize_field("columns", &self.columns)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Family {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "columns",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Columns,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "columns" => Ok(GeneratedField::Columns),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Family;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Family")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Family, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut columns__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Columns => {
                            if columns__.is_some() {
                                return Err(serde::de::Error::duplicate_field("columns"));
                            }
                            columns__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Family {
                    name: name__.unwrap_or_default(),
                    columns: columns__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Family", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FullReadStatsView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.read_iteration_stats.is_some() {
            len += 1;
        }
        if self.request_latency_stats.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.FullReadStatsView", len)?;
        if let Some(v) = self.read_iteration_stats.as_ref() {
            struct_ser.serialize_field("readIterationStats", v)?;
        }
        if let Some(v) = self.request_latency_stats.as_ref() {
            struct_ser.serialize_field("requestLatencyStats", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FullReadStatsView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "read_iteration_stats",
            "readIterationStats",
            "request_latency_stats",
            "requestLatencyStats",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ReadIterationStats,
            RequestLatencyStats,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "readIterationStats" | "read_iteration_stats" => Ok(GeneratedField::ReadIterationStats),
                            "requestLatencyStats" | "request_latency_stats" => Ok(GeneratedField::RequestLatencyStats),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FullReadStatsView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.FullReadStatsView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FullReadStatsView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut read_iteration_stats__ = None;
                let mut request_latency_stats__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ReadIterationStats => {
                            if read_iteration_stats__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readIterationStats"));
                            }
                            read_iteration_stats__ = map.next_value()?;
                        }
                        GeneratedField::RequestLatencyStats => {
                            if request_latency_stats__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestLatencyStats"));
                            }
                            request_latency_stats__ = map.next_value()?;
                        }
                    }
                }
                Ok(FullReadStatsView {
                    read_iteration_stats: read_iteration_stats__,
                    request_latency_stats: request_latency_stats__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.FullReadStatsView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateInitialChangeStreamPartitionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.GenerateInitialChangeStreamPartitionsRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateInitialChangeStreamPartitionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateInitialChangeStreamPartitionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.GenerateInitialChangeStreamPartitionsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenerateInitialChangeStreamPartitionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenerateInitialChangeStreamPartitionsRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.GenerateInitialChangeStreamPartitionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateInitialChangeStreamPartitionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.partition.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.GenerateInitialChangeStreamPartitionsResponse", len)?;
        if let Some(v) = self.partition.as_ref() {
            struct_ser.serialize_field("partition", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateInitialChangeStreamPartitionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "partition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Partition,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "partition" => Ok(GeneratedField::Partition),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateInitialChangeStreamPartitionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.GenerateInitialChangeStreamPartitionsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenerateInitialChangeStreamPartitionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut partition__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Partition => {
                            if partition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partition"));
                            }
                            partition__ = map.next_value()?;
                        }
                    }
                }
                Ok(GenerateInitialChangeStreamPartitionsResponse {
                    partition: partition__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.GenerateInitialChangeStreamPartitionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MutateRowRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        if !self.row_key.is_empty() {
            len += 1;
        }
        if !self.mutations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.MutateRowRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        if !self.row_key.is_empty() {
            struct_ser.serialize_field("rowKey", pbjson::private::base64::encode(&self.row_key).as_str())?;
        }
        if !self.mutations.is_empty() {
            struct_ser.serialize_field("mutations", &self.mutations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MutateRowRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
            "row_key",
            "rowKey",
            "mutations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
            RowKey,
            Mutations,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            "rowKey" | "row_key" => Ok(GeneratedField::RowKey),
                            "mutations" => Ok(GeneratedField::Mutations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MutateRowRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.MutateRowRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MutateRowRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                let mut row_key__ = None;
                let mut mutations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::RowKey => {
                            if row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKey"));
                            }
                            row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Mutations => {
                            if mutations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mutations"));
                            }
                            mutations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MutateRowRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                    row_key: row_key__.unwrap_or_default(),
                    mutations: mutations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.MutateRowRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MutateRowResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("google.bigtable.v2.MutateRowResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MutateRowResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MutateRowResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.MutateRowResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MutateRowResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MutateRowResponse {
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.MutateRowResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MutateRowsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        if !self.entries.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.MutateRowsRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MutateRowsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
            "entries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
            Entries,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            "entries" => Ok(GeneratedField::Entries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MutateRowsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.MutateRowsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MutateRowsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                let mut entries__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MutateRowsRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                    entries: entries__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.MutateRowsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for mutate_rows_request::Entry {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.row_key.is_empty() {
            len += 1;
        }
        if !self.mutations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.MutateRowsRequest.Entry", len)?;
        if !self.row_key.is_empty() {
            struct_ser.serialize_field("rowKey", pbjson::private::base64::encode(&self.row_key).as_str())?;
        }
        if !self.mutations.is_empty() {
            struct_ser.serialize_field("mutations", &self.mutations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for mutate_rows_request::Entry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "row_key",
            "rowKey",
            "mutations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RowKey,
            Mutations,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rowKey" | "row_key" => Ok(GeneratedField::RowKey),
                            "mutations" => Ok(GeneratedField::Mutations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = mutate_rows_request::Entry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.MutateRowsRequest.Entry")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<mutate_rows_request::Entry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut row_key__ = None;
                let mut mutations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RowKey => {
                            if row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKey"));
                            }
                            row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Mutations => {
                            if mutations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mutations"));
                            }
                            mutations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(mutate_rows_request::Entry {
                    row_key: row_key__.unwrap_or_default(),
                    mutations: mutations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.MutateRowsRequest.Entry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MutateRowsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entries.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.MutateRowsResponse", len)?;
        if !self.entries.is_empty() {
            struct_ser.serialize_field("entries", &self.entries)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MutateRowsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "entries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Entries,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "entries" => Ok(GeneratedField::Entries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MutateRowsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.MutateRowsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MutateRowsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut entries__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Entries => {
                            if entries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entries"));
                            }
                            entries__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MutateRowsResponse {
                    entries: entries__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.MutateRowsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for mutate_rows_response::Entry {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.index != 0 {
            len += 1;
        }
        if self.status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.MutateRowsResponse.Entry", len)?;
        if self.index != 0 {
            struct_ser.serialize_field("index", ToString::to_string(&self.index).as_str())?;
        }
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for mutate_rows_response::Entry {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "index",
            "status",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Index,
            Status,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "index" => Ok(GeneratedField::Index),
                            "status" => Ok(GeneratedField::Status),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = mutate_rows_response::Entry;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.MutateRowsResponse.Entry")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<mutate_rows_response::Entry, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut index__ = None;
                let mut status__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Index => {
                            if index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("index"));
                            }
                            index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = map.next_value()?;
                        }
                    }
                }
                Ok(mutate_rows_response::Entry {
                    index: index__.unwrap_or_default(),
                    status: status__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.MutateRowsResponse.Entry", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Mutation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.mutation.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Mutation", len)?;
        if let Some(v) = self.mutation.as_ref() {
            match v {
                mutation::Mutation::SetCell(v) => {
                    struct_ser.serialize_field("setCell", v)?;
                }
                mutation::Mutation::DeleteFromColumn(v) => {
                    struct_ser.serialize_field("deleteFromColumn", v)?;
                }
                mutation::Mutation::DeleteFromFamily(v) => {
                    struct_ser.serialize_field("deleteFromFamily", v)?;
                }
                mutation::Mutation::DeleteFromRow(v) => {
                    struct_ser.serialize_field("deleteFromRow", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Mutation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "set_cell",
            "setCell",
            "delete_from_column",
            "deleteFromColumn",
            "delete_from_family",
            "deleteFromFamily",
            "delete_from_row",
            "deleteFromRow",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SetCell,
            DeleteFromColumn,
            DeleteFromFamily,
            DeleteFromRow,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "setCell" | "set_cell" => Ok(GeneratedField::SetCell),
                            "deleteFromColumn" | "delete_from_column" => Ok(GeneratedField::DeleteFromColumn),
                            "deleteFromFamily" | "delete_from_family" => Ok(GeneratedField::DeleteFromFamily),
                            "deleteFromRow" | "delete_from_row" => Ok(GeneratedField::DeleteFromRow),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Mutation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Mutation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Mutation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mutation__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SetCell => {
                            if mutation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("setCell"));
                            }
                            mutation__ = map.next_value::<::std::option::Option<_>>()?.map(mutation::Mutation::SetCell)
;
                        }
                        GeneratedField::DeleteFromColumn => {
                            if mutation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteFromColumn"));
                            }
                            mutation__ = map.next_value::<::std::option::Option<_>>()?.map(mutation::Mutation::DeleteFromColumn)
;
                        }
                        GeneratedField::DeleteFromFamily => {
                            if mutation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteFromFamily"));
                            }
                            mutation__ = map.next_value::<::std::option::Option<_>>()?.map(mutation::Mutation::DeleteFromFamily)
;
                        }
                        GeneratedField::DeleteFromRow => {
                            if mutation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteFromRow"));
                            }
                            mutation__ = map.next_value::<::std::option::Option<_>>()?.map(mutation::Mutation::DeleteFromRow)
;
                        }
                    }
                }
                Ok(Mutation {
                    mutation: mutation__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Mutation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for mutation::DeleteFromColumn {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.family_name.is_empty() {
            len += 1;
        }
        if !self.column_qualifier.is_empty() {
            len += 1;
        }
        if self.time_range.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Mutation.DeleteFromColumn", len)?;
        if !self.family_name.is_empty() {
            struct_ser.serialize_field("familyName", &self.family_name)?;
        }
        if !self.column_qualifier.is_empty() {
            struct_ser.serialize_field("columnQualifier", pbjson::private::base64::encode(&self.column_qualifier).as_str())?;
        }
        if let Some(v) = self.time_range.as_ref() {
            struct_ser.serialize_field("timeRange", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for mutation::DeleteFromColumn {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "family_name",
            "familyName",
            "column_qualifier",
            "columnQualifier",
            "time_range",
            "timeRange",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FamilyName,
            ColumnQualifier,
            TimeRange,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "familyName" | "family_name" => Ok(GeneratedField::FamilyName),
                            "columnQualifier" | "column_qualifier" => Ok(GeneratedField::ColumnQualifier),
                            "timeRange" | "time_range" => Ok(GeneratedField::TimeRange),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = mutation::DeleteFromColumn;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Mutation.DeleteFromColumn")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<mutation::DeleteFromColumn, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut family_name__ = None;
                let mut column_qualifier__ = None;
                let mut time_range__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FamilyName => {
                            if family_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("familyName"));
                            }
                            family_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnQualifier => {
                            if column_qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnQualifier"));
                            }
                            column_qualifier__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TimeRange => {
                            if time_range__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeRange"));
                            }
                            time_range__ = map.next_value()?;
                        }
                    }
                }
                Ok(mutation::DeleteFromColumn {
                    family_name: family_name__.unwrap_or_default(),
                    column_qualifier: column_qualifier__.unwrap_or_default(),
                    time_range: time_range__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Mutation.DeleteFromColumn", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for mutation::DeleteFromFamily {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.family_name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Mutation.DeleteFromFamily", len)?;
        if !self.family_name.is_empty() {
            struct_ser.serialize_field("familyName", &self.family_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for mutation::DeleteFromFamily {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "family_name",
            "familyName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FamilyName,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "familyName" | "family_name" => Ok(GeneratedField::FamilyName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = mutation::DeleteFromFamily;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Mutation.DeleteFromFamily")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<mutation::DeleteFromFamily, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut family_name__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FamilyName => {
                            if family_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("familyName"));
                            }
                            family_name__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(mutation::DeleteFromFamily {
                    family_name: family_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Mutation.DeleteFromFamily", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for mutation::DeleteFromRow {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("google.bigtable.v2.Mutation.DeleteFromRow", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for mutation::DeleteFromRow {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = mutation::DeleteFromRow;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Mutation.DeleteFromRow")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<mutation::DeleteFromRow, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(mutation::DeleteFromRow {
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Mutation.DeleteFromRow", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for mutation::SetCell {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.family_name.is_empty() {
            len += 1;
        }
        if !self.column_qualifier.is_empty() {
            len += 1;
        }
        if self.timestamp_micros != 0 {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Mutation.SetCell", len)?;
        if !self.family_name.is_empty() {
            struct_ser.serialize_field("familyName", &self.family_name)?;
        }
        if !self.column_qualifier.is_empty() {
            struct_ser.serialize_field("columnQualifier", pbjson::private::base64::encode(&self.column_qualifier).as_str())?;
        }
        if self.timestamp_micros != 0 {
            struct_ser.serialize_field("timestampMicros", ToString::to_string(&self.timestamp_micros).as_str())?;
        }
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", pbjson::private::base64::encode(&self.value).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for mutation::SetCell {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "family_name",
            "familyName",
            "column_qualifier",
            "columnQualifier",
            "timestamp_micros",
            "timestampMicros",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FamilyName,
            ColumnQualifier,
            TimestampMicros,
            Value,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "familyName" | "family_name" => Ok(GeneratedField::FamilyName),
                            "columnQualifier" | "column_qualifier" => Ok(GeneratedField::ColumnQualifier),
                            "timestampMicros" | "timestamp_micros" => Ok(GeneratedField::TimestampMicros),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = mutation::SetCell;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Mutation.SetCell")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<mutation::SetCell, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut family_name__ = None;
                let mut column_qualifier__ = None;
                let mut timestamp_micros__ = None;
                let mut value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FamilyName => {
                            if family_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("familyName"));
                            }
                            family_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnQualifier => {
                            if column_qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnQualifier"));
                            }
                            column_qualifier__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TimestampMicros => {
                            if timestamp_micros__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestampMicros"));
                            }
                            timestamp_micros__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(mutation::SetCell {
                    family_name: family_name__.unwrap_or_default(),
                    column_qualifier: column_qualifier__.unwrap_or_default(),
                    timestamp_micros: timestamp_micros__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Mutation.SetCell", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PingAndWarmRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.PingAndWarmRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PingAndWarmRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "app_profile_id",
            "appProfileId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            AppProfileId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PingAndWarmRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.PingAndWarmRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PingAndWarmRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut app_profile_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PingAndWarmRequest {
                    name: name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.PingAndWarmRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PingAndWarmResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("google.bigtable.v2.PingAndWarmResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PingAndWarmResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PingAndWarmResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.PingAndWarmResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PingAndWarmResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PingAndWarmResponse {
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.PingAndWarmResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReadChangeStreamRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        if self.partition.is_some() {
            len += 1;
        }
        if self.end_time.is_some() {
            len += 1;
        }
        if self.heartbeat_duration.is_some() {
            len += 1;
        }
        if self.start_from.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadChangeStreamRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        if let Some(v) = self.partition.as_ref() {
            struct_ser.serialize_field("partition", v)?;
        }
        if let Some(v) = self.end_time.as_ref() {
            struct_ser.serialize_field("endTime", v)?;
        }
        if let Some(v) = self.heartbeat_duration.as_ref() {
            struct_ser.serialize_field("heartbeatDuration", v)?;
        }
        if let Some(v) = self.start_from.as_ref() {
            match v {
                read_change_stream_request::StartFrom::StartTime(v) => {
                    struct_ser.serialize_field("startTime", v)?;
                }
                read_change_stream_request::StartFrom::ContinuationTokens(v) => {
                    struct_ser.serialize_field("continuationTokens", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadChangeStreamRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
            "partition",
            "end_time",
            "endTime",
            "heartbeat_duration",
            "heartbeatDuration",
            "start_time",
            "startTime",
            "continuation_tokens",
            "continuationTokens",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
            Partition,
            EndTime,
            HeartbeatDuration,
            StartTime,
            ContinuationTokens,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            "partition" => Ok(GeneratedField::Partition),
                            "endTime" | "end_time" => Ok(GeneratedField::EndTime),
                            "heartbeatDuration" | "heartbeat_duration" => Ok(GeneratedField::HeartbeatDuration),
                            "startTime" | "start_time" => Ok(GeneratedField::StartTime),
                            "continuationTokens" | "continuation_tokens" => Ok(GeneratedField::ContinuationTokens),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadChangeStreamRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadChangeStreamRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadChangeStreamRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                let mut partition__ = None;
                let mut end_time__ = None;
                let mut heartbeat_duration__ = None;
                let mut start_from__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Partition => {
                            if partition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partition"));
                            }
                            partition__ = map.next_value()?;
                        }
                        GeneratedField::EndTime => {
                            if end_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endTime"));
                            }
                            end_time__ = map.next_value()?;
                        }
                        GeneratedField::HeartbeatDuration => {
                            if heartbeat_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heartbeatDuration"));
                            }
                            heartbeat_duration__ = map.next_value()?;
                        }
                        GeneratedField::StartTime => {
                            if start_from__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTime"));
                            }
                            start_from__ = map.next_value::<::std::option::Option<_>>()?.map(read_change_stream_request::StartFrom::StartTime)
;
                        }
                        GeneratedField::ContinuationTokens => {
                            if start_from__.is_some() {
                                return Err(serde::de::Error::duplicate_field("continuationTokens"));
                            }
                            start_from__ = map.next_value::<::std::option::Option<_>>()?.map(read_change_stream_request::StartFrom::ContinuationTokens)
;
                        }
                    }
                }
                Ok(ReadChangeStreamRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                    partition: partition__,
                    end_time: end_time__,
                    heartbeat_duration: heartbeat_duration__,
                    start_from: start_from__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadChangeStreamRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReadChangeStreamResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.stream_record.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadChangeStreamResponse", len)?;
        if let Some(v) = self.stream_record.as_ref() {
            match v {
                read_change_stream_response::StreamRecord::DataChange(v) => {
                    struct_ser.serialize_field("dataChange", v)?;
                }
                read_change_stream_response::StreamRecord::Heartbeat(v) => {
                    struct_ser.serialize_field("heartbeat", v)?;
                }
                read_change_stream_response::StreamRecord::CloseStream(v) => {
                    struct_ser.serialize_field("closeStream", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadChangeStreamResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "data_change",
            "dataChange",
            "heartbeat",
            "close_stream",
            "closeStream",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DataChange,
            Heartbeat,
            CloseStream,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "dataChange" | "data_change" => Ok(GeneratedField::DataChange),
                            "heartbeat" => Ok(GeneratedField::Heartbeat),
                            "closeStream" | "close_stream" => Ok(GeneratedField::CloseStream),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadChangeStreamResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadChangeStreamResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadChangeStreamResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut stream_record__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DataChange => {
                            if stream_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataChange"));
                            }
                            stream_record__ = map.next_value::<::std::option::Option<_>>()?.map(read_change_stream_response::StreamRecord::DataChange)
;
                        }
                        GeneratedField::Heartbeat => {
                            if stream_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("heartbeat"));
                            }
                            stream_record__ = map.next_value::<::std::option::Option<_>>()?.map(read_change_stream_response::StreamRecord::Heartbeat)
;
                        }
                        GeneratedField::CloseStream => {
                            if stream_record__.is_some() {
                                return Err(serde::de::Error::duplicate_field("closeStream"));
                            }
                            stream_record__ = map.next_value::<::std::option::Option<_>>()?.map(read_change_stream_response::StreamRecord::CloseStream)
;
                        }
                    }
                }
                Ok(ReadChangeStreamResponse {
                    stream_record: stream_record__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadChangeStreamResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for read_change_stream_response::CloseStream {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.status.is_some() {
            len += 1;
        }
        if !self.continuation_tokens.is_empty() {
            len += 1;
        }
        if !self.new_partitions.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadChangeStreamResponse.CloseStream", len)?;
        if let Some(v) = self.status.as_ref() {
            struct_ser.serialize_field("status", v)?;
        }
        if !self.continuation_tokens.is_empty() {
            struct_ser.serialize_field("continuationTokens", &self.continuation_tokens)?;
        }
        if !self.new_partitions.is_empty() {
            struct_ser.serialize_field("newPartitions", &self.new_partitions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for read_change_stream_response::CloseStream {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "status",
            "continuation_tokens",
            "continuationTokens",
            "new_partitions",
            "newPartitions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Status,
            ContinuationTokens,
            NewPartitions,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "status" => Ok(GeneratedField::Status),
                            "continuationTokens" | "continuation_tokens" => Ok(GeneratedField::ContinuationTokens),
                            "newPartitions" | "new_partitions" => Ok(GeneratedField::NewPartitions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_change_stream_response::CloseStream;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadChangeStreamResponse.CloseStream")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<read_change_stream_response::CloseStream, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut status__ = None;
                let mut continuation_tokens__ = None;
                let mut new_partitions__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = map.next_value()?;
                        }
                        GeneratedField::ContinuationTokens => {
                            if continuation_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("continuationTokens"));
                            }
                            continuation_tokens__ = Some(map.next_value()?);
                        }
                        GeneratedField::NewPartitions => {
                            if new_partitions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newPartitions"));
                            }
                            new_partitions__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(read_change_stream_response::CloseStream {
                    status: status__,
                    continuation_tokens: continuation_tokens__.unwrap_or_default(),
                    new_partitions: new_partitions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadChangeStreamResponse.CloseStream", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for read_change_stream_response::DataChange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type != 0 {
            len += 1;
        }
        if !self.source_cluster_id.is_empty() {
            len += 1;
        }
        if !self.row_key.is_empty() {
            len += 1;
        }
        if self.commit_timestamp.is_some() {
            len += 1;
        }
        if self.tiebreaker != 0 {
            len += 1;
        }
        if !self.chunks.is_empty() {
            len += 1;
        }
        if self.done {
            len += 1;
        }
        if !self.token.is_empty() {
            len += 1;
        }
        if self.estimated_low_watermark.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadChangeStreamResponse.DataChange", len)?;
        if self.r#type != 0 {
            let v = read_change_stream_response::data_change::Type::from_i32(self.r#type)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if !self.source_cluster_id.is_empty() {
            struct_ser.serialize_field("sourceClusterId", &self.source_cluster_id)?;
        }
        if !self.row_key.is_empty() {
            struct_ser.serialize_field("rowKey", pbjson::private::base64::encode(&self.row_key).as_str())?;
        }
        if let Some(v) = self.commit_timestamp.as_ref() {
            struct_ser.serialize_field("commitTimestamp", v)?;
        }
        if self.tiebreaker != 0 {
            struct_ser.serialize_field("tiebreaker", &self.tiebreaker)?;
        }
        if !self.chunks.is_empty() {
            struct_ser.serialize_field("chunks", &self.chunks)?;
        }
        if self.done {
            struct_ser.serialize_field("done", &self.done)?;
        }
        if !self.token.is_empty() {
            struct_ser.serialize_field("token", &self.token)?;
        }
        if let Some(v) = self.estimated_low_watermark.as_ref() {
            struct_ser.serialize_field("estimatedLowWatermark", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for read_change_stream_response::DataChange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "source_cluster_id",
            "sourceClusterId",
            "row_key",
            "rowKey",
            "commit_timestamp",
            "commitTimestamp",
            "tiebreaker",
            "chunks",
            "done",
            "token",
            "estimated_low_watermark",
            "estimatedLowWatermark",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            SourceClusterId,
            RowKey,
            CommitTimestamp,
            Tiebreaker,
            Chunks,
            Done,
            Token,
            EstimatedLowWatermark,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "type" => Ok(GeneratedField::Type),
                            "sourceClusterId" | "source_cluster_id" => Ok(GeneratedField::SourceClusterId),
                            "rowKey" | "row_key" => Ok(GeneratedField::RowKey),
                            "commitTimestamp" | "commit_timestamp" => Ok(GeneratedField::CommitTimestamp),
                            "tiebreaker" => Ok(GeneratedField::Tiebreaker),
                            "chunks" => Ok(GeneratedField::Chunks),
                            "done" => Ok(GeneratedField::Done),
                            "token" => Ok(GeneratedField::Token),
                            "estimatedLowWatermark" | "estimated_low_watermark" => Ok(GeneratedField::EstimatedLowWatermark),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_change_stream_response::DataChange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadChangeStreamResponse.DataChange")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<read_change_stream_response::DataChange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut source_cluster_id__ = None;
                let mut row_key__ = None;
                let mut commit_timestamp__ = None;
                let mut tiebreaker__ = None;
                let mut chunks__ = None;
                let mut done__ = None;
                let mut token__ = None;
                let mut estimated_low_watermark__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map.next_value::<read_change_stream_response::data_change::Type>()? as i32);
                        }
                        GeneratedField::SourceClusterId => {
                            if source_cluster_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sourceClusterId"));
                            }
                            source_cluster_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::RowKey => {
                            if row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKey"));
                            }
                            row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CommitTimestamp => {
                            if commit_timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitTimestamp"));
                            }
                            commit_timestamp__ = map.next_value()?;
                        }
                        GeneratedField::Tiebreaker => {
                            if tiebreaker__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tiebreaker"));
                            }
                            tiebreaker__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Chunks => {
                            if chunks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunks"));
                            }
                            chunks__ = Some(map.next_value()?);
                        }
                        GeneratedField::Done => {
                            if done__.is_some() {
                                return Err(serde::de::Error::duplicate_field("done"));
                            }
                            done__ = Some(map.next_value()?);
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = Some(map.next_value()?);
                        }
                        GeneratedField::EstimatedLowWatermark => {
                            if estimated_low_watermark__.is_some() {
                                return Err(serde::de::Error::duplicate_field("estimatedLowWatermark"));
                            }
                            estimated_low_watermark__ = map.next_value()?;
                        }
                    }
                }
                Ok(read_change_stream_response::DataChange {
                    r#type: r#type__.unwrap_or_default(),
                    source_cluster_id: source_cluster_id__.unwrap_or_default(),
                    row_key: row_key__.unwrap_or_default(),
                    commit_timestamp: commit_timestamp__,
                    tiebreaker: tiebreaker__.unwrap_or_default(),
                    chunks: chunks__.unwrap_or_default(),
                    done: done__.unwrap_or_default(),
                    token: token__.unwrap_or_default(),
                    estimated_low_watermark: estimated_low_watermark__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadChangeStreamResponse.DataChange", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for read_change_stream_response::data_change::Type {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TYPE_UNSPECIFIED",
            Self::User => "USER",
            Self::GarbageCollection => "GARBAGE_COLLECTION",
            Self::Continuation => "CONTINUATION",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for read_change_stream_response::data_change::Type {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TYPE_UNSPECIFIED",
            "USER",
            "GARBAGE_COLLECTION",
            "CONTINUATION",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_change_stream_response::data_change::Type;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(read_change_stream_response::data_change::Type::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(read_change_stream_response::data_change::Type::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TYPE_UNSPECIFIED" => Ok(read_change_stream_response::data_change::Type::Unspecified),
                    "USER" => Ok(read_change_stream_response::data_change::Type::User),
                    "GARBAGE_COLLECTION" => Ok(read_change_stream_response::data_change::Type::GarbageCollection),
                    "CONTINUATION" => Ok(read_change_stream_response::data_change::Type::Continuation),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for read_change_stream_response::Heartbeat {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.continuation_token.is_some() {
            len += 1;
        }
        if self.estimated_low_watermark.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadChangeStreamResponse.Heartbeat", len)?;
        if let Some(v) = self.continuation_token.as_ref() {
            struct_ser.serialize_field("continuationToken", v)?;
        }
        if let Some(v) = self.estimated_low_watermark.as_ref() {
            struct_ser.serialize_field("estimatedLowWatermark", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for read_change_stream_response::Heartbeat {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "continuation_token",
            "continuationToken",
            "estimated_low_watermark",
            "estimatedLowWatermark",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ContinuationToken,
            EstimatedLowWatermark,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "continuationToken" | "continuation_token" => Ok(GeneratedField::ContinuationToken),
                            "estimatedLowWatermark" | "estimated_low_watermark" => Ok(GeneratedField::EstimatedLowWatermark),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_change_stream_response::Heartbeat;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadChangeStreamResponse.Heartbeat")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<read_change_stream_response::Heartbeat, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut continuation_token__ = None;
                let mut estimated_low_watermark__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ContinuationToken => {
                            if continuation_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("continuationToken"));
                            }
                            continuation_token__ = map.next_value()?;
                        }
                        GeneratedField::EstimatedLowWatermark => {
                            if estimated_low_watermark__.is_some() {
                                return Err(serde::de::Error::duplicate_field("estimatedLowWatermark"));
                            }
                            estimated_low_watermark__ = map.next_value()?;
                        }
                    }
                }
                Ok(read_change_stream_response::Heartbeat {
                    continuation_token: continuation_token__,
                    estimated_low_watermark: estimated_low_watermark__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadChangeStreamResponse.Heartbeat", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for read_change_stream_response::MutationChunk {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.chunk_info.is_some() {
            len += 1;
        }
        if self.mutation.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadChangeStreamResponse.MutationChunk", len)?;
        if let Some(v) = self.chunk_info.as_ref() {
            struct_ser.serialize_field("chunkInfo", v)?;
        }
        if let Some(v) = self.mutation.as_ref() {
            struct_ser.serialize_field("mutation", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for read_change_stream_response::MutationChunk {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chunk_info",
            "chunkInfo",
            "mutation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChunkInfo,
            Mutation,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "chunkInfo" | "chunk_info" => Ok(GeneratedField::ChunkInfo),
                            "mutation" => Ok(GeneratedField::Mutation),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_change_stream_response::MutationChunk;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadChangeStreamResponse.MutationChunk")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<read_change_stream_response::MutationChunk, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chunk_info__ = None;
                let mut mutation__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChunkInfo => {
                            if chunk_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunkInfo"));
                            }
                            chunk_info__ = map.next_value()?;
                        }
                        GeneratedField::Mutation => {
                            if mutation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mutation"));
                            }
                            mutation__ = map.next_value()?;
                        }
                    }
                }
                Ok(read_change_stream_response::MutationChunk {
                    chunk_info: chunk_info__,
                    mutation: mutation__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadChangeStreamResponse.MutationChunk", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for read_change_stream_response::mutation_chunk::ChunkInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.chunked_value_size != 0 {
            len += 1;
        }
        if self.chunked_value_offset != 0 {
            len += 1;
        }
        if self.last_chunk {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadChangeStreamResponse.MutationChunk.ChunkInfo", len)?;
        if self.chunked_value_size != 0 {
            struct_ser.serialize_field("chunkedValueSize", &self.chunked_value_size)?;
        }
        if self.chunked_value_offset != 0 {
            struct_ser.serialize_field("chunkedValueOffset", &self.chunked_value_offset)?;
        }
        if self.last_chunk {
            struct_ser.serialize_field("lastChunk", &self.last_chunk)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for read_change_stream_response::mutation_chunk::ChunkInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chunked_value_size",
            "chunkedValueSize",
            "chunked_value_offset",
            "chunkedValueOffset",
            "last_chunk",
            "lastChunk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChunkedValueSize,
            ChunkedValueOffset,
            LastChunk,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "chunkedValueSize" | "chunked_value_size" => Ok(GeneratedField::ChunkedValueSize),
                            "chunkedValueOffset" | "chunked_value_offset" => Ok(GeneratedField::ChunkedValueOffset),
                            "lastChunk" | "last_chunk" => Ok(GeneratedField::LastChunk),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_change_stream_response::mutation_chunk::ChunkInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadChangeStreamResponse.MutationChunk.ChunkInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<read_change_stream_response::mutation_chunk::ChunkInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chunked_value_size__ = None;
                let mut chunked_value_offset__ = None;
                let mut last_chunk__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChunkedValueSize => {
                            if chunked_value_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunkedValueSize"));
                            }
                            chunked_value_size__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ChunkedValueOffset => {
                            if chunked_value_offset__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunkedValueOffset"));
                            }
                            chunked_value_offset__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::LastChunk => {
                            if last_chunk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastChunk"));
                            }
                            last_chunk__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(read_change_stream_response::mutation_chunk::ChunkInfo {
                    chunked_value_size: chunked_value_size__.unwrap_or_default(),
                    chunked_value_offset: chunked_value_offset__.unwrap_or_default(),
                    last_chunk: last_chunk__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadChangeStreamResponse.MutationChunk.ChunkInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReadIterationStats {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.rows_seen_count != 0 {
            len += 1;
        }
        if self.rows_returned_count != 0 {
            len += 1;
        }
        if self.cells_seen_count != 0 {
            len += 1;
        }
        if self.cells_returned_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadIterationStats", len)?;
        if self.rows_seen_count != 0 {
            struct_ser.serialize_field("rowsSeenCount", ToString::to_string(&self.rows_seen_count).as_str())?;
        }
        if self.rows_returned_count != 0 {
            struct_ser.serialize_field("rowsReturnedCount", ToString::to_string(&self.rows_returned_count).as_str())?;
        }
        if self.cells_seen_count != 0 {
            struct_ser.serialize_field("cellsSeenCount", ToString::to_string(&self.cells_seen_count).as_str())?;
        }
        if self.cells_returned_count != 0 {
            struct_ser.serialize_field("cellsReturnedCount", ToString::to_string(&self.cells_returned_count).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadIterationStats {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rows_seen_count",
            "rowsSeenCount",
            "rows_returned_count",
            "rowsReturnedCount",
            "cells_seen_count",
            "cellsSeenCount",
            "cells_returned_count",
            "cellsReturnedCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RowsSeenCount,
            RowsReturnedCount,
            CellsSeenCount,
            CellsReturnedCount,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rowsSeenCount" | "rows_seen_count" => Ok(GeneratedField::RowsSeenCount),
                            "rowsReturnedCount" | "rows_returned_count" => Ok(GeneratedField::RowsReturnedCount),
                            "cellsSeenCount" | "cells_seen_count" => Ok(GeneratedField::CellsSeenCount),
                            "cellsReturnedCount" | "cells_returned_count" => Ok(GeneratedField::CellsReturnedCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadIterationStats;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadIterationStats")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadIterationStats, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rows_seen_count__ = None;
                let mut rows_returned_count__ = None;
                let mut cells_seen_count__ = None;
                let mut cells_returned_count__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RowsSeenCount => {
                            if rows_seen_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowsSeenCount"));
                            }
                            rows_seen_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RowsReturnedCount => {
                            if rows_returned_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowsReturnedCount"));
                            }
                            rows_returned_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CellsSeenCount => {
                            if cells_seen_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellsSeenCount"));
                            }
                            cells_seen_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CellsReturnedCount => {
                            if cells_returned_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellsReturnedCount"));
                            }
                            cells_returned_count__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(ReadIterationStats {
                    rows_seen_count: rows_seen_count__.unwrap_or_default(),
                    rows_returned_count: rows_returned_count__.unwrap_or_default(),
                    cells_seen_count: cells_seen_count__.unwrap_or_default(),
                    cells_returned_count: cells_returned_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadIterationStats", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReadModifyWriteRowRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        if !self.row_key.is_empty() {
            len += 1;
        }
        if !self.rules.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadModifyWriteRowRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        if !self.row_key.is_empty() {
            struct_ser.serialize_field("rowKey", pbjson::private::base64::encode(&self.row_key).as_str())?;
        }
        if !self.rules.is_empty() {
            struct_ser.serialize_field("rules", &self.rules)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadModifyWriteRowRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
            "row_key",
            "rowKey",
            "rules",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
            RowKey,
            Rules,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            "rowKey" | "row_key" => Ok(GeneratedField::RowKey),
                            "rules" => Ok(GeneratedField::Rules),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadModifyWriteRowRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadModifyWriteRowRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadModifyWriteRowRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                let mut row_key__ = None;
                let mut rules__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::RowKey => {
                            if row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKey"));
                            }
                            row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Rules => {
                            if rules__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rules"));
                            }
                            rules__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ReadModifyWriteRowRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                    row_key: row_key__.unwrap_or_default(),
                    rules: rules__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadModifyWriteRowRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReadModifyWriteRowResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.row.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadModifyWriteRowResponse", len)?;
        if let Some(v) = self.row.as_ref() {
            struct_ser.serialize_field("row", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadModifyWriteRowResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "row",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Row,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "row" => Ok(GeneratedField::Row),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadModifyWriteRowResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadModifyWriteRowResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadModifyWriteRowResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut row__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Row => {
                            if row__.is_some() {
                                return Err(serde::de::Error::duplicate_field("row"));
                            }
                            row__ = map.next_value()?;
                        }
                    }
                }
                Ok(ReadModifyWriteRowResponse {
                    row: row__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadModifyWriteRowResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReadModifyWriteRule {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.family_name.is_empty() {
            len += 1;
        }
        if !self.column_qualifier.is_empty() {
            len += 1;
        }
        if self.rule.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadModifyWriteRule", len)?;
        if !self.family_name.is_empty() {
            struct_ser.serialize_field("familyName", &self.family_name)?;
        }
        if !self.column_qualifier.is_empty() {
            struct_ser.serialize_field("columnQualifier", pbjson::private::base64::encode(&self.column_qualifier).as_str())?;
        }
        if let Some(v) = self.rule.as_ref() {
            match v {
                read_modify_write_rule::Rule::AppendValue(v) => {
                    struct_ser.serialize_field("appendValue", pbjson::private::base64::encode(&v).as_str())?;
                }
                read_modify_write_rule::Rule::IncrementAmount(v) => {
                    struct_ser.serialize_field("incrementAmount", ToString::to_string(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadModifyWriteRule {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "family_name",
            "familyName",
            "column_qualifier",
            "columnQualifier",
            "append_value",
            "appendValue",
            "increment_amount",
            "incrementAmount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FamilyName,
            ColumnQualifier,
            AppendValue,
            IncrementAmount,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "familyName" | "family_name" => Ok(GeneratedField::FamilyName),
                            "columnQualifier" | "column_qualifier" => Ok(GeneratedField::ColumnQualifier),
                            "appendValue" | "append_value" => Ok(GeneratedField::AppendValue),
                            "incrementAmount" | "increment_amount" => Ok(GeneratedField::IncrementAmount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadModifyWriteRule;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadModifyWriteRule")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadModifyWriteRule, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut family_name__ = None;
                let mut column_qualifier__ = None;
                let mut rule__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FamilyName => {
                            if family_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("familyName"));
                            }
                            family_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::ColumnQualifier => {
                            if column_qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnQualifier"));
                            }
                            column_qualifier__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AppendValue => {
                            if rule__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appendValue"));
                            }
                            rule__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| read_modify_write_rule::Rule::AppendValue(x.0));
                        }
                        GeneratedField::IncrementAmount => {
                            if rule__.is_some() {
                                return Err(serde::de::Error::duplicate_field("incrementAmount"));
                            }
                            rule__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| read_modify_write_rule::Rule::IncrementAmount(x.0));
                        }
                    }
                }
                Ok(ReadModifyWriteRule {
                    family_name: family_name__.unwrap_or_default(),
                    column_qualifier: column_qualifier__.unwrap_or_default(),
                    rule: rule__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadModifyWriteRule", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ReadRowsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        if self.rows.is_some() {
            len += 1;
        }
        if self.filter.is_some() {
            len += 1;
        }
        if self.rows_limit != 0 {
            len += 1;
        }
        if self.request_stats_view != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadRowsRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        if let Some(v) = self.rows.as_ref() {
            struct_ser.serialize_field("rows", v)?;
        }
        if let Some(v) = self.filter.as_ref() {
            struct_ser.serialize_field("filter", v)?;
        }
        if self.rows_limit != 0 {
            struct_ser.serialize_field("rowsLimit", ToString::to_string(&self.rows_limit).as_str())?;
        }
        if self.request_stats_view != 0 {
            let v = read_rows_request::RequestStatsView::from_i32(self.request_stats_view)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.request_stats_view)))?;
            struct_ser.serialize_field("requestStatsView", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadRowsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
            "rows",
            "filter",
            "rows_limit",
            "rowsLimit",
            "request_stats_view",
            "requestStatsView",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
            Rows,
            Filter,
            RowsLimit,
            RequestStatsView,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            "rows" => Ok(GeneratedField::Rows),
                            "filter" => Ok(GeneratedField::Filter),
                            "rowsLimit" | "rows_limit" => Ok(GeneratedField::RowsLimit),
                            "requestStatsView" | "request_stats_view" => Ok(GeneratedField::RequestStatsView),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadRowsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadRowsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadRowsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                let mut rows__ = None;
                let mut filter__ = None;
                let mut rows_limit__ = None;
                let mut request_stats_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Rows => {
                            if rows__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rows"));
                            }
                            rows__ = map.next_value()?;
                        }
                        GeneratedField::Filter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filter"));
                            }
                            filter__ = map.next_value()?;
                        }
                        GeneratedField::RowsLimit => {
                            if rows_limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowsLimit"));
                            }
                            rows_limit__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RequestStatsView => {
                            if request_stats_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestStatsView"));
                            }
                            request_stats_view__ = Some(map.next_value::<read_rows_request::RequestStatsView>()? as i32);
                        }
                    }
                }
                Ok(ReadRowsRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                    rows: rows__,
                    filter: filter__,
                    rows_limit: rows_limit__.unwrap_or_default(),
                    request_stats_view: request_stats_view__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadRowsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for read_rows_request::RequestStatsView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "REQUEST_STATS_VIEW_UNSPECIFIED",
            Self::RequestStatsNone => "REQUEST_STATS_NONE",
            Self::RequestStatsFull => "REQUEST_STATS_FULL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for read_rows_request::RequestStatsView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "REQUEST_STATS_VIEW_UNSPECIFIED",
            "REQUEST_STATS_NONE",
            "REQUEST_STATS_FULL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_rows_request::RequestStatsView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(read_rows_request::RequestStatsView::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(read_rows_request::RequestStatsView::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "REQUEST_STATS_VIEW_UNSPECIFIED" => Ok(read_rows_request::RequestStatsView::Unspecified),
                    "REQUEST_STATS_NONE" => Ok(read_rows_request::RequestStatsView::RequestStatsNone),
                    "REQUEST_STATS_FULL" => Ok(read_rows_request::RequestStatsView::RequestStatsFull),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ReadRowsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.chunks.is_empty() {
            len += 1;
        }
        if !self.last_scanned_row_key.is_empty() {
            len += 1;
        }
        if self.request_stats.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadRowsResponse", len)?;
        if !self.chunks.is_empty() {
            struct_ser.serialize_field("chunks", &self.chunks)?;
        }
        if !self.last_scanned_row_key.is_empty() {
            struct_ser.serialize_field("lastScannedRowKey", pbjson::private::base64::encode(&self.last_scanned_row_key).as_str())?;
        }
        if let Some(v) = self.request_stats.as_ref() {
            struct_ser.serialize_field("requestStats", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadRowsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chunks",
            "last_scanned_row_key",
            "lastScannedRowKey",
            "request_stats",
            "requestStats",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Chunks,
            LastScannedRowKey,
            RequestStats,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "chunks" => Ok(GeneratedField::Chunks),
                            "lastScannedRowKey" | "last_scanned_row_key" => Ok(GeneratedField::LastScannedRowKey),
                            "requestStats" | "request_stats" => Ok(GeneratedField::RequestStats),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadRowsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadRowsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ReadRowsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chunks__ = None;
                let mut last_scanned_row_key__ = None;
                let mut request_stats__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Chunks => {
                            if chunks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chunks"));
                            }
                            chunks__ = Some(map.next_value()?);
                        }
                        GeneratedField::LastScannedRowKey => {
                            if last_scanned_row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastScannedRowKey"));
                            }
                            last_scanned_row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::RequestStats => {
                            if request_stats__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestStats"));
                            }
                            request_stats__ = map.next_value()?;
                        }
                    }
                }
                Ok(ReadRowsResponse {
                    chunks: chunks__.unwrap_or_default(),
                    last_scanned_row_key: last_scanned_row_key__.unwrap_or_default(),
                    request_stats: request_stats__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadRowsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for read_rows_response::CellChunk {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.row_key.is_empty() {
            len += 1;
        }
        if self.family_name.is_some() {
            len += 1;
        }
        if self.qualifier.is_some() {
            len += 1;
        }
        if self.timestamp_micros != 0 {
            len += 1;
        }
        if !self.labels.is_empty() {
            len += 1;
        }
        if !self.value.is_empty() {
            len += 1;
        }
        if self.value_size != 0 {
            len += 1;
        }
        if self.row_status.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ReadRowsResponse.CellChunk", len)?;
        if !self.row_key.is_empty() {
            struct_ser.serialize_field("rowKey", pbjson::private::base64::encode(&self.row_key).as_str())?;
        }
        if let Some(v) = self.family_name.as_ref() {
            struct_ser.serialize_field("familyName", v)?;
        }
        if let Some(v) = self.qualifier.as_ref() {
            struct_ser.serialize_field("qualifier", v)?;
        }
        if self.timestamp_micros != 0 {
            struct_ser.serialize_field("timestampMicros", ToString::to_string(&self.timestamp_micros).as_str())?;
        }
        if !self.labels.is_empty() {
            struct_ser.serialize_field("labels", &self.labels)?;
        }
        if !self.value.is_empty() {
            struct_ser.serialize_field("value", pbjson::private::base64::encode(&self.value).as_str())?;
        }
        if self.value_size != 0 {
            struct_ser.serialize_field("valueSize", &self.value_size)?;
        }
        if let Some(v) = self.row_status.as_ref() {
            match v {
                read_rows_response::cell_chunk::RowStatus::ResetRow(v) => {
                    struct_ser.serialize_field("resetRow", v)?;
                }
                read_rows_response::cell_chunk::RowStatus::CommitRow(v) => {
                    struct_ser.serialize_field("commitRow", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for read_rows_response::CellChunk {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "row_key",
            "rowKey",
            "family_name",
            "familyName",
            "qualifier",
            "timestamp_micros",
            "timestampMicros",
            "labels",
            "value",
            "value_size",
            "valueSize",
            "reset_row",
            "resetRow",
            "commit_row",
            "commitRow",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RowKey,
            FamilyName,
            Qualifier,
            TimestampMicros,
            Labels,
            Value,
            ValueSize,
            ResetRow,
            CommitRow,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rowKey" | "row_key" => Ok(GeneratedField::RowKey),
                            "familyName" | "family_name" => Ok(GeneratedField::FamilyName),
                            "qualifier" => Ok(GeneratedField::Qualifier),
                            "timestampMicros" | "timestamp_micros" => Ok(GeneratedField::TimestampMicros),
                            "labels" => Ok(GeneratedField::Labels),
                            "value" => Ok(GeneratedField::Value),
                            "valueSize" | "value_size" => Ok(GeneratedField::ValueSize),
                            "resetRow" | "reset_row" => Ok(GeneratedField::ResetRow),
                            "commitRow" | "commit_row" => Ok(GeneratedField::CommitRow),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = read_rows_response::CellChunk;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ReadRowsResponse.CellChunk")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<read_rows_response::CellChunk, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut row_key__ = None;
                let mut family_name__ = None;
                let mut qualifier__ = None;
                let mut timestamp_micros__ = None;
                let mut labels__ = None;
                let mut value__ = None;
                let mut value_size__ = None;
                let mut row_status__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RowKey => {
                            if row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKey"));
                            }
                            row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FamilyName => {
                            if family_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("familyName"));
                            }
                            family_name__ = map.next_value()?;
                        }
                        GeneratedField::Qualifier => {
                            if qualifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("qualifier"));
                            }
                            qualifier__ = map.next_value()?;
                        }
                        GeneratedField::TimestampMicros => {
                            if timestamp_micros__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestampMicros"));
                            }
                            timestamp_micros__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Labels => {
                            if labels__.is_some() {
                                return Err(serde::de::Error::duplicate_field("labels"));
                            }
                            labels__ = Some(map.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValueSize => {
                            if value_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueSize"));
                            }
                            value_size__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ResetRow => {
                            if row_status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("resetRow"));
                            }
                            row_status__ = map.next_value::<::std::option::Option<_>>()?.map(read_rows_response::cell_chunk::RowStatus::ResetRow);
                        }
                        GeneratedField::CommitRow => {
                            if row_status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitRow"));
                            }
                            row_status__ = map.next_value::<::std::option::Option<_>>()?.map(read_rows_response::cell_chunk::RowStatus::CommitRow);
                        }
                    }
                }
                Ok(read_rows_response::CellChunk {
                    row_key: row_key__.unwrap_or_default(),
                    family_name: family_name__,
                    qualifier: qualifier__,
                    timestamp_micros: timestamp_micros__.unwrap_or_default(),
                    labels: labels__.unwrap_or_default(),
                    value: value__.unwrap_or_default(),
                    value_size: value_size__.unwrap_or_default(),
                    row_status: row_status__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ReadRowsResponse.CellChunk", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RequestLatencyStats {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.frontend_server_latency.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RequestLatencyStats", len)?;
        if let Some(v) = self.frontend_server_latency.as_ref() {
            struct_ser.serialize_field("frontendServerLatency", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RequestLatencyStats {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "frontend_server_latency",
            "frontendServerLatency",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FrontendServerLatency,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "frontendServerLatency" | "frontend_server_latency" => Ok(GeneratedField::FrontendServerLatency),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RequestLatencyStats;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RequestLatencyStats")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RequestLatencyStats, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut frontend_server_latency__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FrontendServerLatency => {
                            if frontend_server_latency__.is_some() {
                                return Err(serde::de::Error::duplicate_field("frontendServerLatency"));
                            }
                            frontend_server_latency__ = map.next_value()?;
                        }
                    }
                }
                Ok(RequestLatencyStats {
                    frontend_server_latency: frontend_server_latency__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RequestLatencyStats", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RequestStats {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.stats_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RequestStats", len)?;
        if let Some(v) = self.stats_view.as_ref() {
            match v {
                request_stats::StatsView::FullReadStatsView(v) => {
                    struct_ser.serialize_field("fullReadStatsView", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RequestStats {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "full_read_stats_view",
            "fullReadStatsView",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FullReadStatsView,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "fullReadStatsView" | "full_read_stats_view" => Ok(GeneratedField::FullReadStatsView),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RequestStats;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RequestStats")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RequestStats, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut stats_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::FullReadStatsView => {
                            if stats_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullReadStatsView"));
                            }
                            stats_view__ = map.next_value::<::std::option::Option<_>>()?.map(request_stats::StatsView::FullReadStatsView)
;
                        }
                    }
                }
                Ok(RequestStats {
                    stats_view: stats_view__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RequestStats", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Row {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.key.is_empty() {
            len += 1;
        }
        if !self.families.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.Row", len)?;
        if !self.key.is_empty() {
            struct_ser.serialize_field("key", pbjson::private::base64::encode(&self.key).as_str())?;
        }
        if !self.families.is_empty() {
            struct_ser.serialize_field("families", &self.families)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Row {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "key",
            "families",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Key,
            Families,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "key" => Ok(GeneratedField::Key),
                            "families" => Ok(GeneratedField::Families),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Row;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.Row")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Row, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut key__ = None;
                let mut families__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Families => {
                            if families__.is_some() {
                                return Err(serde::de::Error::duplicate_field("families"));
                            }
                            families__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Row {
                    key: key__.unwrap_or_default(),
                    families: families__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.Row", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RowFilter {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.filter.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RowFilter", len)?;
        if let Some(v) = self.filter.as_ref() {
            match v {
                row_filter::Filter::Chain(v) => {
                    struct_ser.serialize_field("chain", v)?;
                }
                row_filter::Filter::Interleave(v) => {
                    struct_ser.serialize_field("interleave", v)?;
                }
                row_filter::Filter::Condition(v) => {
                    struct_ser.serialize_field("condition", v)?;
                }
                row_filter::Filter::Sink(v) => {
                    struct_ser.serialize_field("sink", v)?;
                }
                row_filter::Filter::PassAllFilter(v) => {
                    struct_ser.serialize_field("passAllFilter", v)?;
                }
                row_filter::Filter::BlockAllFilter(v) => {
                    struct_ser.serialize_field("blockAllFilter", v)?;
                }
                row_filter::Filter::RowKeyRegexFilter(v) => {
                    struct_ser.serialize_field("rowKeyRegexFilter", pbjson::private::base64::encode(&v).as_str())?;
                }
                row_filter::Filter::RowSampleFilter(v) => {
                    struct_ser.serialize_field("rowSampleFilter", v)?;
                }
                row_filter::Filter::FamilyNameRegexFilter(v) => {
                    struct_ser.serialize_field("familyNameRegexFilter", v)?;
                }
                row_filter::Filter::ColumnQualifierRegexFilter(v) => {
                    struct_ser.serialize_field("columnQualifierRegexFilter", pbjson::private::base64::encode(&v).as_str())?;
                }
                row_filter::Filter::ColumnRangeFilter(v) => {
                    struct_ser.serialize_field("columnRangeFilter", v)?;
                }
                row_filter::Filter::TimestampRangeFilter(v) => {
                    struct_ser.serialize_field("timestampRangeFilter", v)?;
                }
                row_filter::Filter::ValueRegexFilter(v) => {
                    struct_ser.serialize_field("valueRegexFilter", pbjson::private::base64::encode(&v).as_str())?;
                }
                row_filter::Filter::ValueRangeFilter(v) => {
                    struct_ser.serialize_field("valueRangeFilter", v)?;
                }
                row_filter::Filter::CellsPerRowOffsetFilter(v) => {
                    struct_ser.serialize_field("cellsPerRowOffsetFilter", v)?;
                }
                row_filter::Filter::CellsPerRowLimitFilter(v) => {
                    struct_ser.serialize_field("cellsPerRowLimitFilter", v)?;
                }
                row_filter::Filter::CellsPerColumnLimitFilter(v) => {
                    struct_ser.serialize_field("cellsPerColumnLimitFilter", v)?;
                }
                row_filter::Filter::StripValueTransformer(v) => {
                    struct_ser.serialize_field("stripValueTransformer", v)?;
                }
                row_filter::Filter::ApplyLabelTransformer(v) => {
                    struct_ser.serialize_field("applyLabelTransformer", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RowFilter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain",
            "interleave",
            "condition",
            "sink",
            "pass_all_filter",
            "passAllFilter",
            "block_all_filter",
            "blockAllFilter",
            "row_key_regex_filter",
            "rowKeyRegexFilter",
            "row_sample_filter",
            "rowSampleFilter",
            "family_name_regex_filter",
            "familyNameRegexFilter",
            "column_qualifier_regex_filter",
            "columnQualifierRegexFilter",
            "column_range_filter",
            "columnRangeFilter",
            "timestamp_range_filter",
            "timestampRangeFilter",
            "value_regex_filter",
            "valueRegexFilter",
            "value_range_filter",
            "valueRangeFilter",
            "cells_per_row_offset_filter",
            "cellsPerRowOffsetFilter",
            "cells_per_row_limit_filter",
            "cellsPerRowLimitFilter",
            "cells_per_column_limit_filter",
            "cellsPerColumnLimitFilter",
            "strip_value_transformer",
            "stripValueTransformer",
            "apply_label_transformer",
            "applyLabelTransformer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Chain,
            Interleave,
            Condition,
            Sink,
            PassAllFilter,
            BlockAllFilter,
            RowKeyRegexFilter,
            RowSampleFilter,
            FamilyNameRegexFilter,
            ColumnQualifierRegexFilter,
            ColumnRangeFilter,
            TimestampRangeFilter,
            ValueRegexFilter,
            ValueRangeFilter,
            CellsPerRowOffsetFilter,
            CellsPerRowLimitFilter,
            CellsPerColumnLimitFilter,
            StripValueTransformer,
            ApplyLabelTransformer,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "chain" => Ok(GeneratedField::Chain),
                            "interleave" => Ok(GeneratedField::Interleave),
                            "condition" => Ok(GeneratedField::Condition),
                            "sink" => Ok(GeneratedField::Sink),
                            "passAllFilter" | "pass_all_filter" => Ok(GeneratedField::PassAllFilter),
                            "blockAllFilter" | "block_all_filter" => Ok(GeneratedField::BlockAllFilter),
                            "rowKeyRegexFilter" | "row_key_regex_filter" => Ok(GeneratedField::RowKeyRegexFilter),
                            "rowSampleFilter" | "row_sample_filter" => Ok(GeneratedField::RowSampleFilter),
                            "familyNameRegexFilter" | "family_name_regex_filter" => Ok(GeneratedField::FamilyNameRegexFilter),
                            "columnQualifierRegexFilter" | "column_qualifier_regex_filter" => Ok(GeneratedField::ColumnQualifierRegexFilter),
                            "columnRangeFilter" | "column_range_filter" => Ok(GeneratedField::ColumnRangeFilter),
                            "timestampRangeFilter" | "timestamp_range_filter" => Ok(GeneratedField::TimestampRangeFilter),
                            "valueRegexFilter" | "value_regex_filter" => Ok(GeneratedField::ValueRegexFilter),
                            "valueRangeFilter" | "value_range_filter" => Ok(GeneratedField::ValueRangeFilter),
                            "cellsPerRowOffsetFilter" | "cells_per_row_offset_filter" => Ok(GeneratedField::CellsPerRowOffsetFilter),
                            "cellsPerRowLimitFilter" | "cells_per_row_limit_filter" => Ok(GeneratedField::CellsPerRowLimitFilter),
                            "cellsPerColumnLimitFilter" | "cells_per_column_limit_filter" => Ok(GeneratedField::CellsPerColumnLimitFilter),
                            "stripValueTransformer" | "strip_value_transformer" => Ok(GeneratedField::StripValueTransformer),
                            "applyLabelTransformer" | "apply_label_transformer" => Ok(GeneratedField::ApplyLabelTransformer),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RowFilter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RowFilter")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RowFilter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut filter__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Chain => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chain"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::Chain)
;
                        }
                        GeneratedField::Interleave => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interleave"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::Interleave)
;
                        }
                        GeneratedField::Condition => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("condition"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::Condition)
;
                        }
                        GeneratedField::Sink => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sink"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::Sink);
                        }
                        GeneratedField::PassAllFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("passAllFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::PassAllFilter);
                        }
                        GeneratedField::BlockAllFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockAllFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::BlockAllFilter);
                        }
                        GeneratedField::RowKeyRegexFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKeyRegexFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| row_filter::Filter::RowKeyRegexFilter(x.0));
                        }
                        GeneratedField::RowSampleFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowSampleFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| row_filter::Filter::RowSampleFilter(x.0));
                        }
                        GeneratedField::FamilyNameRegexFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("familyNameRegexFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::FamilyNameRegexFilter);
                        }
                        GeneratedField::ColumnQualifierRegexFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnQualifierRegexFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| row_filter::Filter::ColumnQualifierRegexFilter(x.0));
                        }
                        GeneratedField::ColumnRangeFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnRangeFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::ColumnRangeFilter)
;
                        }
                        GeneratedField::TimestampRangeFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestampRangeFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::TimestampRangeFilter)
;
                        }
                        GeneratedField::ValueRegexFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueRegexFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| row_filter::Filter::ValueRegexFilter(x.0));
                        }
                        GeneratedField::ValueRangeFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueRangeFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::ValueRangeFilter)
;
                        }
                        GeneratedField::CellsPerRowOffsetFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellsPerRowOffsetFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| row_filter::Filter::CellsPerRowOffsetFilter(x.0));
                        }
                        GeneratedField::CellsPerRowLimitFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellsPerRowLimitFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| row_filter::Filter::CellsPerRowLimitFilter(x.0));
                        }
                        GeneratedField::CellsPerColumnLimitFilter => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cellsPerColumnLimitFilter"));
                            }
                            filter__ = map.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| row_filter::Filter::CellsPerColumnLimitFilter(x.0));
                        }
                        GeneratedField::StripValueTransformer => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stripValueTransformer"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::StripValueTransformer);
                        }
                        GeneratedField::ApplyLabelTransformer => {
                            if filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("applyLabelTransformer"));
                            }
                            filter__ = map.next_value::<::std::option::Option<_>>()?.map(row_filter::Filter::ApplyLabelTransformer);
                        }
                    }
                }
                Ok(RowFilter {
                    filter: filter__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RowFilter", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for row_filter::Chain {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.filters.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RowFilter.Chain", len)?;
        if !self.filters.is_empty() {
            struct_ser.serialize_field("filters", &self.filters)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for row_filter::Chain {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "filters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Filters,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "filters" => Ok(GeneratedField::Filters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = row_filter::Chain;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RowFilter.Chain")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<row_filter::Chain, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut filters__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Filters => {
                            if filters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filters"));
                            }
                            filters__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(row_filter::Chain {
                    filters: filters__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RowFilter.Chain", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for row_filter::Condition {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.predicate_filter.is_some() {
            len += 1;
        }
        if self.true_filter.is_some() {
            len += 1;
        }
        if self.false_filter.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RowFilter.Condition", len)?;
        if let Some(v) = self.predicate_filter.as_ref() {
            struct_ser.serialize_field("predicateFilter", v)?;
        }
        if let Some(v) = self.true_filter.as_ref() {
            struct_ser.serialize_field("trueFilter", v)?;
        }
        if let Some(v) = self.false_filter.as_ref() {
            struct_ser.serialize_field("falseFilter", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for row_filter::Condition {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "predicate_filter",
            "predicateFilter",
            "true_filter",
            "trueFilter",
            "false_filter",
            "falseFilter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PredicateFilter,
            TrueFilter,
            FalseFilter,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "predicateFilter" | "predicate_filter" => Ok(GeneratedField::PredicateFilter),
                            "trueFilter" | "true_filter" => Ok(GeneratedField::TrueFilter),
                            "falseFilter" | "false_filter" => Ok(GeneratedField::FalseFilter),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = row_filter::Condition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RowFilter.Condition")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<row_filter::Condition, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut predicate_filter__ = None;
                let mut true_filter__ = None;
                let mut false_filter__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PredicateFilter => {
                            if predicate_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("predicateFilter"));
                            }
                            predicate_filter__ = map.next_value()?;
                        }
                        GeneratedField::TrueFilter => {
                            if true_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trueFilter"));
                            }
                            true_filter__ = map.next_value()?;
                        }
                        GeneratedField::FalseFilter => {
                            if false_filter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("falseFilter"));
                            }
                            false_filter__ = map.next_value()?;
                        }
                    }
                }
                Ok(row_filter::Condition {
                    predicate_filter: predicate_filter__,
                    true_filter: true_filter__,
                    false_filter: false_filter__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RowFilter.Condition", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for row_filter::Interleave {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.filters.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RowFilter.Interleave", len)?;
        if !self.filters.is_empty() {
            struct_ser.serialize_field("filters", &self.filters)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for row_filter::Interleave {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "filters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Filters,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "filters" => Ok(GeneratedField::Filters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = row_filter::Interleave;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RowFilter.Interleave")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<row_filter::Interleave, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut filters__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Filters => {
                            if filters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filters"));
                            }
                            filters__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(row_filter::Interleave {
                    filters: filters__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RowFilter.Interleave", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RowRange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_key.is_some() {
            len += 1;
        }
        if self.end_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RowRange", len)?;
        if let Some(v) = self.start_key.as_ref() {
            match v {
                row_range::StartKey::StartKeyClosed(v) => {
                    struct_ser.serialize_field("startKeyClosed", pbjson::private::base64::encode(&v).as_str())?;
                }
                row_range::StartKey::StartKeyOpen(v) => {
                    struct_ser.serialize_field("startKeyOpen", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        if let Some(v) = self.end_key.as_ref() {
            match v {
                row_range::EndKey::EndKeyOpen(v) => {
                    struct_ser.serialize_field("endKeyOpen", pbjson::private::base64::encode(&v).as_str())?;
                }
                row_range::EndKey::EndKeyClosed(v) => {
                    struct_ser.serialize_field("endKeyClosed", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RowRange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_key_closed",
            "startKeyClosed",
            "start_key_open",
            "startKeyOpen",
            "end_key_open",
            "endKeyOpen",
            "end_key_closed",
            "endKeyClosed",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartKeyClosed,
            StartKeyOpen,
            EndKeyOpen,
            EndKeyClosed,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "startKeyClosed" | "start_key_closed" => Ok(GeneratedField::StartKeyClosed),
                            "startKeyOpen" | "start_key_open" => Ok(GeneratedField::StartKeyOpen),
                            "endKeyOpen" | "end_key_open" => Ok(GeneratedField::EndKeyOpen),
                            "endKeyClosed" | "end_key_closed" => Ok(GeneratedField::EndKeyClosed),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RowRange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RowRange")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RowRange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_key__ = None;
                let mut end_key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::StartKeyClosed => {
                            if start_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startKeyClosed"));
                            }
                            start_key__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| row_range::StartKey::StartKeyClosed(x.0));
                        }
                        GeneratedField::StartKeyOpen => {
                            if start_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startKeyOpen"));
                            }
                            start_key__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| row_range::StartKey::StartKeyOpen(x.0));
                        }
                        GeneratedField::EndKeyOpen => {
                            if end_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endKeyOpen"));
                            }
                            end_key__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| row_range::EndKey::EndKeyOpen(x.0));
                        }
                        GeneratedField::EndKeyClosed => {
                            if end_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endKeyClosed"));
                            }
                            end_key__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| row_range::EndKey::EndKeyClosed(x.0));
                        }
                    }
                }
                Ok(RowRange {
                    start_key: start_key__,
                    end_key: end_key__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RowRange", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RowSet {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.row_keys.is_empty() {
            len += 1;
        }
        if !self.row_ranges.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.RowSet", len)?;
        if !self.row_keys.is_empty() {
            struct_ser.serialize_field("rowKeys", &self.row_keys.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        if !self.row_ranges.is_empty() {
            struct_ser.serialize_field("rowRanges", &self.row_ranges)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RowSet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "row_keys",
            "rowKeys",
            "row_ranges",
            "rowRanges",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RowKeys,
            RowRanges,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rowKeys" | "row_keys" => Ok(GeneratedField::RowKeys),
                            "rowRanges" | "row_ranges" => Ok(GeneratedField::RowRanges),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RowSet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.RowSet")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RowSet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut row_keys__ = None;
                let mut row_ranges__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RowKeys => {
                            if row_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKeys"));
                            }
                            row_keys__ = 
                                Some(map.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::RowRanges => {
                            if row_ranges__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowRanges"));
                            }
                            row_ranges__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RowSet {
                    row_keys: row_keys__.unwrap_or_default(),
                    row_ranges: row_ranges__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.RowSet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SampleRowKeysRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_name.is_empty() {
            len += 1;
        }
        if !self.app_profile_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.SampleRowKeysRequest", len)?;
        if !self.table_name.is_empty() {
            struct_ser.serialize_field("tableName", &self.table_name)?;
        }
        if !self.app_profile_id.is_empty() {
            struct_ser.serialize_field("appProfileId", &self.app_profile_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SampleRowKeysRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_name",
            "tableName",
            "app_profile_id",
            "appProfileId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableName,
            AppProfileId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tableName" | "table_name" => Ok(GeneratedField::TableName),
                            "appProfileId" | "app_profile_id" => Ok(GeneratedField::AppProfileId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SampleRowKeysRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.SampleRowKeysRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SampleRowKeysRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_name__ = None;
                let mut app_profile_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::TableName => {
                            if table_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableName"));
                            }
                            table_name__ = Some(map.next_value()?);
                        }
                        GeneratedField::AppProfileId => {
                            if app_profile_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appProfileId"));
                            }
                            app_profile_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SampleRowKeysRequest {
                    table_name: table_name__.unwrap_or_default(),
                    app_profile_id: app_profile_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.SampleRowKeysRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SampleRowKeysResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.row_key.is_empty() {
            len += 1;
        }
        if self.offset_bytes != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.SampleRowKeysResponse", len)?;
        if !self.row_key.is_empty() {
            struct_ser.serialize_field("rowKey", pbjson::private::base64::encode(&self.row_key).as_str())?;
        }
        if self.offset_bytes != 0 {
            struct_ser.serialize_field("offsetBytes", ToString::to_string(&self.offset_bytes).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SampleRowKeysResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "row_key",
            "rowKey",
            "offset_bytes",
            "offsetBytes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RowKey,
            OffsetBytes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rowKey" | "row_key" => Ok(GeneratedField::RowKey),
                            "offsetBytes" | "offset_bytes" => Ok(GeneratedField::OffsetBytes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SampleRowKeysResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.SampleRowKeysResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SampleRowKeysResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut row_key__ = None;
                let mut offset_bytes__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RowKey => {
                            if row_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowKey"));
                            }
                            row_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::OffsetBytes => {
                            if offset_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("offsetBytes"));
                            }
                            offset_bytes__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SampleRowKeysResponse {
                    row_key: row_key__.unwrap_or_default(),
                    offset_bytes: offset_bytes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.SampleRowKeysResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamContinuationToken {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.partition.is_some() {
            len += 1;
        }
        if !self.token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.StreamContinuationToken", len)?;
        if let Some(v) = self.partition.as_ref() {
            struct_ser.serialize_field("partition", v)?;
        }
        if !self.token.is_empty() {
            struct_ser.serialize_field("token", &self.token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamContinuationToken {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "partition",
            "token",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Partition,
            Token,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "partition" => Ok(GeneratedField::Partition),
                            "token" => Ok(GeneratedField::Token),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamContinuationToken;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.StreamContinuationToken")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamContinuationToken, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut partition__ = None;
                let mut token__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Partition => {
                            if partition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("partition"));
                            }
                            partition__ = map.next_value()?;
                        }
                        GeneratedField::Token => {
                            if token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("token"));
                            }
                            token__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(StreamContinuationToken {
                    partition: partition__,
                    token: token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.StreamContinuationToken", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamContinuationTokens {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tokens.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.StreamContinuationTokens", len)?;
        if !self.tokens.is_empty() {
            struct_ser.serialize_field("tokens", &self.tokens)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamContinuationTokens {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tokens",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tokens,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tokens" => Ok(GeneratedField::Tokens),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamContinuationTokens;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.StreamContinuationTokens")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamContinuationTokens, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tokens__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Tokens => {
                            if tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tokens"));
                            }
                            tokens__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(StreamContinuationTokens {
                    tokens: tokens__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.StreamContinuationTokens", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StreamPartition {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.row_range.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.StreamPartition", len)?;
        if let Some(v) = self.row_range.as_ref() {
            struct_ser.serialize_field("rowRange", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StreamPartition {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "row_range",
            "rowRange",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RowRange,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rowRange" | "row_range" => Ok(GeneratedField::RowRange),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StreamPartition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.StreamPartition")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StreamPartition, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut row_range__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RowRange => {
                            if row_range__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowRange"));
                            }
                            row_range__ = map.next_value()?;
                        }
                    }
                }
                Ok(StreamPartition {
                    row_range: row_range__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.StreamPartition", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TimestampRange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_timestamp_micros != 0 {
            len += 1;
        }
        if self.end_timestamp_micros != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.TimestampRange", len)?;
        if self.start_timestamp_micros != 0 {
            struct_ser.serialize_field("startTimestampMicros", ToString::to_string(&self.start_timestamp_micros).as_str())?;
        }
        if self.end_timestamp_micros != 0 {
            struct_ser.serialize_field("endTimestampMicros", ToString::to_string(&self.end_timestamp_micros).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TimestampRange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_timestamp_micros",
            "startTimestampMicros",
            "end_timestamp_micros",
            "endTimestampMicros",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartTimestampMicros,
            EndTimestampMicros,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "startTimestampMicros" | "start_timestamp_micros" => Ok(GeneratedField::StartTimestampMicros),
                            "endTimestampMicros" | "end_timestamp_micros" => Ok(GeneratedField::EndTimestampMicros),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TimestampRange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.TimestampRange")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TimestampRange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_timestamp_micros__ = None;
                let mut end_timestamp_micros__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::StartTimestampMicros => {
                            if start_timestamp_micros__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startTimestampMicros"));
                            }
                            start_timestamp_micros__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndTimestampMicros => {
                            if end_timestamp_micros__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endTimestampMicros"));
                            }
                            end_timestamp_micros__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TimestampRange {
                    start_timestamp_micros: start_timestamp_micros__.unwrap_or_default(),
                    end_timestamp_micros: end_timestamp_micros__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.TimestampRange", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ValueRange {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_value.is_some() {
            len += 1;
        }
        if self.end_value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.bigtable.v2.ValueRange", len)?;
        if let Some(v) = self.start_value.as_ref() {
            match v {
                value_range::StartValue::StartValueClosed(v) => {
                    struct_ser.serialize_field("startValueClosed", pbjson::private::base64::encode(&v).as_str())?;
                }
                value_range::StartValue::StartValueOpen(v) => {
                    struct_ser.serialize_field("startValueOpen", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        if let Some(v) = self.end_value.as_ref() {
            match v {
                value_range::EndValue::EndValueClosed(v) => {
                    struct_ser.serialize_field("endValueClosed", pbjson::private::base64::encode(&v).as_str())?;
                }
                value_range::EndValue::EndValueOpen(v) => {
                    struct_ser.serialize_field("endValueOpen", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ValueRange {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_value_closed",
            "startValueClosed",
            "start_value_open",
            "startValueOpen",
            "end_value_closed",
            "endValueClosed",
            "end_value_open",
            "endValueOpen",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartValueClosed,
            StartValueOpen,
            EndValueClosed,
            EndValueOpen,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "startValueClosed" | "start_value_closed" => Ok(GeneratedField::StartValueClosed),
                            "startValueOpen" | "start_value_open" => Ok(GeneratedField::StartValueOpen),
                            "endValueClosed" | "end_value_closed" => Ok(GeneratedField::EndValueClosed),
                            "endValueOpen" | "end_value_open" => Ok(GeneratedField::EndValueOpen),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ValueRange;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.bigtable.v2.ValueRange")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ValueRange, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_value__ = None;
                let mut end_value__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::StartValueClosed => {
                            if start_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startValueClosed"));
                            }
                            start_value__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| value_range::StartValue::StartValueClosed(x.0));
                        }
                        GeneratedField::StartValueOpen => {
                            if start_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startValueOpen"));
                            }
                            start_value__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| value_range::StartValue::StartValueOpen(x.0));
                        }
                        GeneratedField::EndValueClosed => {
                            if end_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endValueClosed"));
                            }
                            end_value__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| value_range::EndValue::EndValueClosed(x.0));
                        }
                        GeneratedField::EndValueOpen => {
                            if end_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endValueOpen"));
                            }
                            end_value__ = map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| value_range::EndValue::EndValueOpen(x.0));
                        }
                    }
                }
                Ok(ValueRange {
                    start_value: start_value__,
                    end_value: end_value__,
                })
            }
        }
        deserializer.deserialize_struct("google.bigtable.v2.ValueRange", FIELDS, GeneratedVisitor)
    }
}
