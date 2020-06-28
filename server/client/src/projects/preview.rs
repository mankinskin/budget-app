use crate::{
    page,
    projects::*,
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
#[derive(Clone)]
pub enum Msg {
    Project(project::Msg),
    Open,
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
                    simple_ev(Ev::Click, Msg::Project(project::Msg::Delete)),
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
