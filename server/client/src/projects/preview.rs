use crate::{
    page,
    projects::*,
};
use database::{
    Entry,
};

#[derive(Clone)]
pub struct Model {
    pub project: project::Model,
}
impl From<project::Model> for Model {
    fn from(model: project::Model) -> Self {
        Self {
            project: model,
        }
    }
}
impl From<&Entry<Project>> for Model {
    fn from(entry: &Entry<Project>) -> Self {
        Self {
            project: project::Model::from(entry),
        }
    }
}
#[derive(Clone)]
pub enum Msg {
    Project(project::Msg),
    Deleted(Result<Option<Project>, String>),
    Open,
    Delete,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::Project(msg) => {
            project::update(
                msg,
                &mut model.project,
                &mut orders.proxy(Msg::Project)
            )
        },
        Msg::Open => {
            page::go_to(project::Config::from(model.project.clone()), orders);
        },
        Msg::Deleted(res) => {
        },
        Msg::Delete => {
            orders.perform_cmd(
                api::delete_project(model.project.project_id)
                .map(|res| Msg::Deleted(res.map_err(|e| format!("{:?}", e))))
            );
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match &model.project.project {
        Some(project) => {
            div![
                a![
                    attrs!{
                        At::Href => "";
                    },
                    project.name(),
                    simple_ev(Ev::Click, Msg::Open),
                ],
                p!["Preview"],
                button![
                    simple_ev(Ev::Click, Msg::Delete),
                    "Delete"
                ],
            ]
        },
        None => {
            div![
                h1!["Preview"],
                p!["Loading..."],
            ]
        },
    }
}
