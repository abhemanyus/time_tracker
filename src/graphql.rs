pub mod queries {
    #[cynic::query_module(
        schema_path = "schema.graphql",
        query_module = "query_dsl",
    )]
    pub mod clients_with_projects {
        use crate::graphql::query_dsl;

        ///```graphql
        ///{
        ///    queryClient {
        ///        id
        ///        name
        ///        projects {
        ///            id
        ///            name
        ///        }
        ///    }
        ///}
        ///```
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct Query {
            pub query_client: Option<Vec<Option<Client>>>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Client")]
        pub struct Client {
            pub id: String,
            pub name: String,
            pub projects: Vec<Project>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Project")]
        pub struct Project {
            pub id: String,
            pub name: String,
        }
    }

    #[cynic::query_module(
        schema_path = "schema.graphql",
        query_module = "query_dsl",
    )]
    pub mod clients_with_projects_with_time_entries {
        use crate::graphql::{query_dsl, types::*};

        ///```graphql
        ///{
        ///    queryClient {
        ///        id
        ///        name
        ///        projects {
        ///            id
        ///            name
        ///            time_entries {
        ///                id
        ///                name
        ///                started
        ///                stopped
        ///            }
        ///        }
        ///    }
        ///}
        ///```
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct Query {
            pub query_client: Option<Vec<Option<Client>>>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Client")]
        pub struct Client {
            pub id: String,
            pub name: String,
            pub projects: Vec<Project>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Project")]
        pub struct Project {
            pub id: String,
            pub name: String,
            pub time_entries: Vec<TimeEntry>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "TimeEntry")]
        pub struct TimeEntry {
            pub id: String,
            pub name: String,
            pub started: DateTime,
            pub stopped: Option<DateTime>,
        }
    }

    #[cynic::query_module(
        schema_path = "schema.graphql",
        query_module = "query_dsl",
    )]
    pub mod clients_with_time_blocks_and_time_entries {
        use crate::graphql::{query_dsl, types::*};

        ///```graphql
        ///{
        ///    queryClient {
        ///        id
        ///        name
        ///        time_blocks {
        ///            id
        ///            name
        ///            status
        ///            duration
        ///            invoice {
        ///                id
        ///                custom_id
        ///                url
        ///            }
        ///        }
        ///        projects {
        ///            time_entries {
        ///                started
        ///                stopped
        ///            }
        ///        }
        ///    }
        ///}
        ///```
        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct Query {
            pub query_client: Option<Vec<Option<Client>>>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Client")]
        pub struct Client {
            pub id: String,
            pub name: String,
            pub time_blocks: Vec<TimeBlock>,
            pub projects: Vec<Project>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "TimeBlock")]
        pub struct TimeBlock {
            pub id: String,
            pub name: String,
            pub status: TimeBlockStatus,
            pub duration: i32,
            pub invoice: Option<Invoice>,
        }

        #[derive(cynic::Enum, Debug, Copy, Clone)]
        #[cynic(graphql_type = "TimeBlockStatus", rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum TimeBlockStatus {
            NonBillable,
            Unpaid,
            Paid,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Invoice")]
        pub struct Invoice {
            pub id: String,
            pub custom_id: Option<String>,
            pub url: Option<String>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Project")]
        pub struct Project {
            pub time_entries: Vec<TimeEntry>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "TimeEntry")]
        pub struct TimeEntry {
            pub started: DateTime,
            pub stopped: Option<DateTime>,
        }
    }
}

mod types {
    #[derive(cynic::Scalar, Debug)]
    pub struct DateTime(pub String);
}

mod query_dsl {
    use super::types::*;
    cynic::query_dsl!("schema.graphql");
}