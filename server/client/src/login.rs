use seed::{
    *,
    prelude::*,
    browser::service::fetch::{
        FailReason,
    },
};
use futures::{
    Future,
};
use plans::{
    credentials::*,
    user::*,
};
use crate::{
    route::{
        self,
        Route,
    },
    root::{
        self,
        GMsg,
    },
};
/// credential input component
#[derive(Clone, Default)]
pub struct Model {
    credentials: Credentials,
    submit_result: Option<Result<(), FailReason<UserSession>>>,
}
impl Model {
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}
#[derive(Clone)]
pub enum Msg {
    ChangeUsername(String),
    ChangePassword(String),
    LoginResponse(ResponseDataResult<UserSession>),
    Login,
    Register,
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg, GMsg>) {
    match msg {
        Msg::ChangeUsername(u) => model.credentials.username = u,
        Msg::ChangePassword(p) => model.credentials.password = p,
        Msg::Login => {
            seed::log!("Logging in...");
            orders.perform_cmd(login_request(model.credentials()));
        },
        Msg::LoginResponse(res) => {
            model.submit_result = Some(res.clone().map(|_| ()));
            match res {
                Ok(session) => {
                    root::set_session(session, orders);
                    route::change_route(Route::Home, orders);
                },
                Err(reason) => {
                    seed::log!(reason);
                },
            }
        },
        Msg::Register => {
            route::change_route(Route::Register, orders);
        },
    }
}
fn login_request(credentials: &Credentials)
    -> impl Future<Output = Result<Msg, Msg>>
{
    Request::new("http://localhost:8000/users/login")
        .method(Method::Post)
        .send_json(credentials)
        .fetch_json_data(move |data_result: ResponseDataResult<UserSession>| {
            Msg::LoginResponse(data_result)
        })
}
pub fn view(model: &Model) -> Node<Msg> {
    // login form
    form![
        // Username field
        label![
            "Username"
        ],
        input![
            attrs!{
                At::Placeholder => "Username",
                At::Value => model.credentials.username,
            },
            input_ev(Ev::Input, Msg::ChangeUsername)
        ],
        div![
            model.credentials.username_invalid_text()
        ],
        // Password field
        label![
            "Password"
        ],
        input![
            attrs!{
                At::Type => "password",
                At::Placeholder => "Password",
                At::Value => model.credentials.password,
            },
            input_ev(Ev::Input, Msg::ChangePassword)
        ],
        div![
            model.credentials.password_invalid_text()
        ],
        // Login Button
        button![
            attrs!{
                At::Type => "submit",
            },
            "Login"
        ],
        ev(Ev::Submit, |ev| {
            ev.prevent_default();
            Msg::Login
        }),
        // Register Button
        button![simple_ev(Ev::Click, Msg::Register), "Register"],
        if let Some(res) = &model.submit_result {
            p![format!("{:?}", res)]
        } else { empty![] }
    ]
}
