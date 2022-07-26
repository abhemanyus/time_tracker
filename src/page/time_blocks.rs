use seed::{prelude::*, *};

use chrono::{prelude::*, Duration};
use ulid::Ulid;

use cynic::QueryBuilder;

use std::collections::BTreeMap;
use std::convert::identity;
use std::ops::Add;

use crate::graphql;

const PRIMARY_COLOR: &str = "#00d1b2";

type ClientId = Ulid;
type InvoiceId = Ulid;
type TimeBlockId = Ulid;

// ------ ------
//     Init
// ------ ------

pub fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::ClientsFetched(request_clients().await) });
    Model {
        changes_status: ChangesStatus::NoChanges,
        errors: Vec::new(),

        clients: RemoteData::Loading,
    }
}

async fn request_clients() -> graphql::Result<BTreeMap<ClientId, Client>> {
    use graphql::queries::clients_with_time_blocks_and_time_entries as query_mod;

    let invoice_mapper = |invoice: query_mod::Invoice| Invoice {
        id: invoice.id.parse().expect("parse invoice Ulid"),
        custom_id: invoice.custom_id,
        url: invoice.url,
    };

    let status_mapper = |status: query_mod::TimeBlockStatus| match status {
        query_mod::TimeBlockStatus::NonBillable => TimeBlockStatus::NonBillable,
        query_mod::TimeBlockStatus::Unpaid => TimeBlockStatus::Unpaid,
        query_mod::TimeBlockStatus::Paid => TimeBlockStatus::Paid,
    };

    let time_block_mapper = |time_block: query_mod::TimeBlock| {
        (
            time_block.id.parse().expect("parse time_block Ulid"),
            TimeBlock {
                name: time_block.name,
                status: status_mapper(time_block.status),
                duration: Duration::seconds(i64::from(time_block.duration)),
                duration_change: None,
                invoice: time_block.invoice.map(invoice_mapper),
            },
        )
    };

    let compute_tracked_time = |projects: Vec<query_mod::Project>| {
        projects
            .into_iter()
            .flat_map(|project| project.time_entries)
            .map(|time_entry| {
                let started: DateTime<Local> = time_entry
                    .started
                    .0
                    .parse()
                    .expect("parse time_entry started");

                let stopped: DateTime<Local> = if let Some(stopped) = time_entry.stopped {
                    stopped.0.parse().expect("parse time_entry stopped")
                } else {
                    chrono::Local::now()
                };

                stopped - started
            })
            .fold(Duration::seconds(0), Duration::add)
    };

    let client_mapper = |client: query_mod::Client| {
        (
            client.id.parse().expect("parse client Ulid"),
            Client {
                name: client.name,
                time_blocks: client
                    .time_blocks
                    .into_iter()
                    .map(time_block_mapper)
                    .collect(),
                tracked: compute_tracked_time(client.projects),
            },
        )
    };

    Ok(graphql::send_operation(query_mod::Query::build(()))
        .await?
        .query_client
        .expect("get clients")
        .into_iter()
        .filter_map(identity)
        .map(client_mapper)
        .collect())
}

// ------ ------
//     Model
// ------ ------

pub struct Model {
    changes_status: ChangesStatus,
    errors: Vec<graphql::GraphQLError>,

    clients: RemoteData<BTreeMap<ClientId, Client>>,
}

enum RemoteData<T> {
    NotAsked,
    Loading,
    Loaded(T),
}

impl<T> RemoteData<T> {
    fn loaded(&self) -> Option<&T> {
        if let Self::Loaded(data) = self {
            Some(data)
        } else {
            None
        }
    }

    fn loaded_mut(&mut self) -> Option<&mut T> {
        if let Self::Loaded(data) = self {
            Some(data)
        } else {
            None
        }
    }
}

enum ChangesStatus {
    NoChanges,
    Saving { requests_in_flight: usize },
    Saved(DateTime<Local>),
}

#[derive(Debug)]
pub struct Client {
    name: String,
    time_blocks: BTreeMap<TimeBlockId, TimeBlock>,
    tracked: Duration,
}

#[derive(Debug)]
struct TimeBlock {
    name: String,
    status: TimeBlockStatus,
    duration: Duration,
    duration_change: Option<String>,
    invoice: Option<Invoice>,
}

#[derive(Debug, Copy, Clone)]
pub enum TimeBlockStatus {
    NonBillable,
    Unpaid,
    Paid,
}

#[derive(Debug)]
struct Invoice {
    id: InvoiceId,
    custom_id: Option<String>,
    url: Option<String>,
}

// ------ ------
//    Update
// ------ ------

pub enum Msg {
    ClientsFetched(graphql::Result<BTreeMap<ClientId, Client>>),
    ChangesSaved(Option<FetchError>),
    ClearErrors,

    // ------ TimeBlock ------
    AddTimeBlock(ClientId),
    DeleteTimeBlock(ClientId, TimeBlockId),
    SetTimeBlockStatus(ClientId, TimeBlockId, TimeBlockStatus),

    TimeBlockNameChanged(ClientId, TimeBlockId, String),
    SaveTimeBlockName(ClientId, TimeBlockId),

    TimeBlockDurationChanged(ClientId, TimeBlockId, String),
    SaveTimeBlockDuration(ClientId, TimeBlockId),

    // ------ Invoice ------
    AttachInvoice(ClientId, TimeBlockId),
    DeleteInvoice(ClientId, TimeBlockId),

    InvoiceCustomIdChanged(ClientId, TimeBlockId, String),
    SaveInvoiceCustomId(ClientId, TimeBlockId),

    InvoiceUrlChanged(ClientId, TimeBlockId, String),
    SaveInvoiceUrl(ClientId, TimeBlockId),
}

pub fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClientsFetched(Ok(clients)) => {
            log!("Msg::ClientsFetched", clients);
            model.clients = RemoteData::Loaded(clients);
        }
        Msg::ClientsFetched(Err(graphql_error)) => {
            model.errors.push(graphql_error);
        }

        Msg::ChangesSaved(None) => {
            log!("Msg::ChangesSaved");
        }
        Msg::ChangesSaved(Some(fetch_error)) => {
            log!("Msg::ChangesSaved", fetch_error);
        }

        Msg::ClearErrors => {}

        // ------ TimeBlock ------
        Msg::AddTimeBlock(client_id) => {
            log!("Msg::AddTimeBlock", client_id);
        }
        Msg::DeleteTimeBlock(client_id, time_block_id) => {
            log!("Msg::DeleteTimeBlock", client_id, time_block_id);
        }
        Msg::SetTimeBlockStatus(client_id, time_block_id, time_block_status) => {
            log!(
                "Msg::SetTimeBlockStatus",
                client_id,
                time_block_id,
                time_block_status
            );
        }

        Msg::TimeBlockNameChanged(client_id, time_block_id, name) => {
            let mut set_time_block_name = move |name| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .time_blocks
                        .get_mut(&time_block_id)?
                        .name = name,
                )
            };
            log!("Msg::TimeBlockNameChanged", client_id, time_block_id, name);
            set_time_block_name(name);
        }
        Msg::SaveTimeBlockName(client_id, time_block_id) => {
            log!("Msg::SaveTimeBlockName", client_id, time_block_id);
        }

        Msg::TimeBlockDurationChanged(client_id, time_block_id, duration) => {
            let mut set_time_block_duration_change = move |duration| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .time_blocks
                        .get_mut(&time_block_id)?
                        .duration_change = Some(duration),
                )
            };
            log!(
                "Msg::TimeBlockDurationChanged",
                client_id,
                time_block_id,
                duration
            );
            set_time_block_duration_change(duration);
        }
        Msg::SaveTimeBlockDuration(client_id, time_block_id) => {
            let mut set_time_block_duration_change = move || -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .time_blocks
                        .get_mut(&time_block_id)?
                        .duration_change = None,
                )
            };
            log!("Msg::SaveTimeBlockDuration", client_id, time_block_id);
            set_time_block_duration_change();
        }

        // ------ Invoice ------
        Msg::AttachInvoice(client_id, time_block_id) => {
            log!("Msg::AttachInvoice", client_id, time_block_id);
        }
        Msg::DeleteInvoice(client_id, time_block_id) => {
            log!("Msg::DeleteInvoice", client_id, time_block_id);
        }

        Msg::InvoiceCustomIdChanged(client_id, time_block_id, custom_id) => {
            let mut set_invoice_custom_id = move |custom_id| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .time_blocks
                        .get_mut(&time_block_id)?
                        .invoice
                        .as_mut()?
                        .custom_id = Some(custom_id),
                )
            };
            log!(
                "Msg::InvoiceCustomIdChanged",
                client_id,
                time_block_id,
                custom_id
            );
            set_invoice_custom_id(custom_id);
        }
        Msg::SaveInvoiceCustomId(client_id, time_block_id) => {
            log!("Msg::SaveInvoiceCustomId", client_id, time_block_id);
        }

        Msg::InvoiceUrlChanged(client_id, time_block_id, url) => {
            let mut set_invoice_url = move |url| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .time_blocks
                        .get_mut(&time_block_id)?
                        .invoice
                        .as_mut()?
                        .url = Some(url),
                )
            };
            log!("Msg::InvoiceUrlChanged", client_id, time_block_id, url);
            set_invoice_url(url);
        }
        Msg::SaveInvoiceUrl(client_id, time_block_id) => {
            log!("Msg::SaveInvoiceUrl", client_id, time_block_id);
        }
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> Node<Msg> {
    div!["TimeBlocks view"]
}
