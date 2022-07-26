use seed::{prelude::*, *};

use chrono::prelude::*;
use ulid::Ulid;

use cynic::QueryBuilder;

use std::collections::BTreeMap;
use std::convert::identity;

use crate::graphql;

const PRIMARY_COLOR: &str = "#00d1b2";
const LINK_COLOR: &str = "#3273dc";

type ClientId = Ulid;
type ProjectId = Ulid;
type TimeEntryId = Ulid;

// ------ ------
//     Init
// ------ ------

pub fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::ClientsFetched(request_clients().await) });

    Model {
        changes_status: ChangesStatus::NoChanges,
        errors: Vec::new(),

        clients: RemoteData::Loading,
        timer_handle: orders.stream_with_handle(streams::interval(1000, || Msg::OnSecondTick)),
    }
}

async fn request_clients() -> graphql::Result<BTreeMap<ClientId, Client>> {
    use graphql::queries::clients_with_projects_with_time_entries as query_mod;

    let time_entry_mapper = |time_entry: query_mod::TimeEntry| {
        (
            time_entry.id.parse().expect("parse time_entry Ulid"),
            TimeEntry {
                name: time_entry.name,
                started: time_entry
                    .started
                    .0
                    .parse()
                    .expect("parse time_entry started time"),
                stopped: time_entry
                    .stopped
                    .map(|time| time.0.parse().expect("parse time_entry started time")),
                change: None,
            },
        )
    };

    let project_mapper = |project: query_mod::Project| {
        (
            project.id.parse().expect("parse project Ulid"),
            Project {
                name: project.name,
                time_entries: project
                    .time_entries
                    .into_iter()
                    .map(time_entry_mapper)
                    .collect(),
            },
        )
    };

    let client_mapper = |client: query_mod::Client| {
        (
            client.id.parse().expect("parse client Ulid"),
            Client {
                name: client.name,
                projects: client.projects.into_iter().map(project_mapper).collect(),
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
    timer_handle: StreamHandle,
}

// ----- Remote Data -----
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
    projects: BTreeMap<Ulid, Project>,
}

#[derive(Debug)]
struct Project {
    name: String,
    time_entries: BTreeMap<Ulid, TimeEntry>,
}

#[derive(Debug)]
struct TimeEntry {
    name: String,
    started: DateTime<Local>,
    stopped: Option<DateTime<Local>>,
    change: Option<TimeEntryChange>,
}

#[derive(Debug)]
enum TimeEntryChange {
    StartedDate(String),
    StartedTime(String),
    StoppedDate(String),
    StoppedTime(String),
    Duration(String),
}

// ------ ------
//    Update
// ------ ------

pub enum Msg {
    ClientsFetched(graphql::Result<BTreeMap<ClientId, Client>>),
    ChangesSaved(Option<FetchError>),
    ClearErrors,

    Start(ClientId, ProjectId),
    Stop(ClientId, ProjectId),

    DeleteTimeEntry(ClientId, ProjectId, TimeEntryId),

    TimeEntryNameChanged(ClientId, ProjectId, TimeEntryId, String),
    SaveTimeEntryName(ClientId, ProjectId, TimeEntryId),

    TimeEntryStartedChanged(ClientId, ProjectId, TimeEntryId, String),
    SaveTimeEntryStarted(ClientId, ProjectId, TimeEntryId),

    TimeEntryDurationChanged(ClientId, ProjectId, TimeEntryId, String),

    TimeEntryStoppedChanged(ClientId, ProjectId, TimeEntryId, String),
    SaveTimeEntryStopped(ClientId, ProjectId, TimeEntryId),

    TimeEntryStartedDateChanged(ClientId, ProjectId, TimeEntryId, String),
    TimeEntryStartedTimeChanged(ClientId, ProjectId, TimeEntryId, String),

    TimeEntryStoppedDateChanged(ClientId, ProjectId, TimeEntryId, String),
    TimeEntryStoppedTimeChanged(ClientId, ProjectId, TimeEntryId, String),

    SaveTimeEntryChange(ClientId, ProjectId, TimeEntryId),

    OnSecondTick,
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

        Msg::ClearErrors => {
            log!("Msg::ClearErrors");
        }

        Msg::Start(client_id, project_id) => {
            log!("Msg::Start", client_id, project_id);
        }
        Msg::Stop(client_id, project_id) => {
            log!("Msg::Stop", client_id, project_id);
        }

        Msg::DeleteTimeEntry(client_id, project_id, time_entry_id) => {
            log!("Msg::DeleteTimeEntry", client_id, project_id, time_entry_id);
        }

        Msg::TimeEntryNameChanged(client_id, project_id, time_entry_id, name) => {
            let mut set_time_entry_name = move |name| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .projects
                        .get_mut(&project_id)?
                        .time_entries
                        .get_mut(&time_entry_id)?
                        .name = name,
                )
            };
            log!(
                "Msg::TimeEntryNameChanged",
                client_id,
                project_id,
                time_entry_id,
                name
            );
            set_time_entry_name(name);
        }
        Msg::SaveTimeEntryName(client_id, project_id, time_entry_id) => {
            log!(
                "Msg::SaveTimeEntryName",
                client_id,
                project_id,
                time_entry_id
            );
        }

        Msg::TimeEntryStartedChanged(client_id, project_id, time_entry_id, started) => {}
        Msg::SaveTimeEntryStarted(client_id, project_id, time_entry_id) => {}

        Msg::TimeEntryStoppedChanged(client_id, project_id, time_entry_id, stopped) => {}
        Msg::SaveTimeEntryStopped(client_id, project_id, time_entry_id) => {}
        Msg::TimeEntryStartedDateChanged(client_id, project_id, time_entry_id, date) => {
            let mut set_time_entry_change = move |change| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .projects
                        .get_mut(&project_id)?
                        .time_entries
                        .get_mut(&time_entry_id)?
                        .change = Some(change),
                )
            };
            log!(
                "Msg::TimeEntryStartedDateChanged",
                client_id,
                project_id,
                time_entry_id,
                date
            );
            set_time_entry_change(TimeEntryChange::StartedDate(date));
        }
        Msg::TimeEntryStartedTimeChanged(client_id, project_id, time_entry_id, time) => {
            let mut set_time_entry_change = move |change| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .projects
                        .get_mut(&project_id)?
                        .time_entries
                        .get_mut(&time_entry_id)?
                        .change = Some(change),
                )
            };
            log!(
                "Msg::TimeEntryStartedTimeChanged",
                client_id,
                project_id,
                time_entry_id,
                time
            );
            set_time_entry_change(TimeEntryChange::StartedTime(time));
        }

        Msg::TimeEntryDurationChanged(client_id, project_id, time_entry_id, duration) => {
            let mut set_time_entry_change = move |change| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .projects
                        .get_mut(&project_id)?
                        .time_entries
                        .get_mut(&time_entry_id)?
                        .change = Some(change),
                )
            };
            log!(
                "Msg::TimeEntryDurationChanged",
                client_id,
                project_id,
                time_entry_id,
                duration
            );
            set_time_entry_change(TimeEntryChange::Duration(duration));
        }

        Msg::TimeEntryStoppedDateChanged(client_id, project_id, time_entry_id, date) => {
            let mut set_time_entry_change = move |change| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .projects
                        .get_mut(&project_id)?
                        .time_entries
                        .get_mut(&time_entry_id)?
                        .change = Some(change),
                )
            };
            log!(
                "Msg::TimeEntryStoppedDateChanged",
                client_id,
                project_id,
                time_entry_id,
                date
            );
            set_time_entry_change(TimeEntryChange::StoppedDate(date));
        }
        Msg::TimeEntryStoppedTimeChanged(client_id, project_id, time_entry_id, time) => {
            let mut set_time_entry_change = move |change| -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .projects
                        .get_mut(&project_id)?
                        .time_entries
                        .get_mut(&time_entry_id)?
                        .change = Some(change),
                )
            };
            log!(
                "Msg::TimeEntryStoppedTimeChanged",
                client_id,
                project_id,
                time_entry_id,
                time
            );
            set_time_entry_change(TimeEntryChange::StoppedTime(time));
        }

        Msg::SaveTimeEntryChange(client_id, project_id, time_entry_id) => {
            let mut delete_time_entry_change = move || -> Option<()> {
                Some(
                    model
                        .clients
                        .loaded_mut()?
                        .get_mut(&client_id)?
                        .projects
                        .get_mut(&project_id)?
                        .time_entries
                        .get_mut(&time_entry_id)?
                        .change = None,
                )
            };
            log!(
                "Msg::SaveTimeEntryChange",
                client_id,
                project_id,
                time_entry_id
            );
            delete_time_entry_change();
        }

        Msg::OnSecondTick => {}
    }
}

// ------ ------
//     View
// ------ ------

pub fn view(model: &Model) -> Node<Msg> {
    div!["TimeTrackerview"]
}
