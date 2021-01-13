use crate::{
    binance::{
        binance,
        BinanceActor,
        PriceHistoryRequest,
    },
    telegram::TelegramActor,
    *,
};
use app_model::{
    auth::{
        login,
        register,
        Credentials,
    },
    user::User,
};
use async_std::net::SocketAddr;
use chrono::Utc;
use shared::{
    PriceSubscription,
};
use tide::{
    Body,
    Endpoint,
    Middleware,
    Request,
    Response,
};
use tide_rustls::TlsListener;
use tide_tracing::TraceMiddleware;
use tide_websockets::{
    WebSocket,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};
use database::Schema;
use database_table::{
    Database,
    DatabaseTable,
    TableRoutable,
};
use enum_paths::AsPath;
use std::fmt::Debug;

macro_rules! client_file {
    ($path:expr) => {
        |_: tide::Request<()>| {
            async move {
                let body = tide::Body::from_file(format!("{}/{}", CLIENT_PATH, $path)).await?;
                Ok(tide::Response::from(body))
            }
        }
    };
}
macro_rules! index {
    () => {
        client_file!("index.html")
    };
}
#[async_trait::async_trait]
trait ServeSession<'db, DB>
    where DB: Database<'db, User> + 'db,
{
    type Api;
    type Response;
    type Request;
    fn serve(api: &mut Self::Api);
    async fn login_handler(mut req: Self::Request) -> Self::Response;
    async fn logout_handler(mut req: Self::Request) -> Self::Response;
    async fn registration_handler(mut req: Self::Request) -> Self::Response;
}
#[async_trait::async_trait]
trait ServeTable<'db, T, DB>
    where T: TableRoutable + DatabaseTable<'db, DB> + 'db,
          DB: Database<'db, T> + 'db,
{
    type Api;
    type Response;
    type Request;
    fn serve(api: &mut Self::Api);
    async fn post_handler(req: Self::Request) -> Self::Response;
    async fn get_handler(req: Self::Request) -> Self::Response;
    async fn get_list_handler(req: Self::Request) -> Self::Response;
    async fn delete_handler(req: Self::Request) -> Self::Response;
}
struct TideServer(tide::Server<()>);

#[async_trait::async_trait]
impl<DB> ServeSession<'static, DB> for TideServer
    where DB: Database<'static, User> + 'static,
{
    type Api = tide::Server<()>;
    type Response = tide::Result;
    type Request = Request<()>;
    fn serve(server: &mut Self::Api) {
        let mut auth = tide::new();
        auth.at("/login").post(<Self as ServeSession<'static, DB>>::login_handler);
        auth.at("/register").post(<Self as ServeSession<'static, DB>>::registration_handler);
        auth.at("/logout").post(<Self as ServeSession<'static, DB>>::logout_handler);
        server.at("/auth").nest(auth);
    }
    async fn login_handler(mut req: Self::Request) -> Self::Response {
        let credentials: Credentials = req.body_json().await?;
        match login::<DB>(credentials).await {
            Ok(session) => {
                req.session_mut()
                    .insert("session", session)
                    .map(|_| Response::new(200))
                    .map_err(|e| tide::Error::from_str(500, e.to_string()))
            }
            Err(e) => Err(e),
        }
    }
    async fn logout_handler(mut req: Self::Request) -> Self::Response {
        req.session_mut().remove("session");
        Ok(Response::new(200))
    }
    async fn registration_handler(mut req: Self::Request) -> Self::Response {
        let user: User = req.body_json().await?;
        match register::<database::Schema>(user).await {
            Ok(_session) => Ok(Response::new(200)),
            Err(e) => Err(tide::Error::from_str(500, e.to_string())),
        }
    }
}

#[async_trait::async_trait]
impl<T, DB> ServeTable<'static, T, DB> for TideServer
    where T: TableRoutable + DatabaseTable<'static, DB> + Debug + 'static,
          DB: Database<'static, T> + 'static,
{
    type Api = tide::Server<()>;
    type Response = tide::Result<Body>;
    type Request = Request<()>;

    fn serve(api: &mut Self::Api) {
        let mut t = tide::new();
        t.at("/")
            .get(<Self as ServeTable<'static, T, DB>>::get_list_handler)
            .post(<Self as ServeTable<'static, T, DB>>::post_handler)
            .delete(<Self as ServeTable<'static, T, DB>>::delete_handler);
        t.at("/:id")
            .get(<Self as ServeTable<'static, T, DB>>::get_handler);
        let route = T::route().as_path();
        debug!("Serving {}", route);
        api.at(&format!("/{}", route)).nest(t);
    }
    async fn post_handler(mut req: Self::Request) -> Self::Response {
        let s: T = req.body_json().await?;
        let id = T::insert(s);
        let body = Body::from_json(&id)?;
        debug!("{:#?}", body);
        Ok(body)
    }
    async fn get_handler(req: Self::Request) -> Self::Response {
        let id: rql::Id<T> = req.param("id")?.parse()?;
        let r = T::get(id);
        Ok(Body::from_json(&r)?)
    }
    async fn get_list_handler(_req: Self::Request) -> Self::Response {
        debug!("Get subscription list handler");
        let list = T::get_all();
        debug!("Result: {:?}", list);
        Ok(Body::from_json(&list)?)
    }
    async fn delete_handler(req: Self::Request) -> Self::Response {
        let id: rql::Id<T> = req.param("id")?.parse()?;
        let r = T::delete(id);
        Ok(Body::from_json(&r)?)
    }
}

impl TideServer {
    pub fn new() -> Self {
        let mut server = tide::new();
        server.with(TraceMiddleware::new());
        server.with(Self::session_middleware());
        server.with(Self::session_validator_middleware());
        server.at("/wss").get(Self::wss_handler);
        server.at("/api").nest(Self::api());
        Self::root(&mut server);
        Self(server)
    }
    fn serve_table<T, DB>(server: &mut tide::Server<()>)
        where T: TableRoutable + Debug + DatabaseTable<'static, DB> + 'static,
              DB: Database<'static, T> + 'static,
    {
        <TideServer as ServeTable<'_, T, DB>>::serve(server)
    }
    fn auth(server: &mut tide::Server<()>) {
        <TideServer as ServeSession<'_, Schema>>::serve(server)
    }
    fn root(server: &mut tide::Server<()>) {
        server.at("/subscriptions").get(index!());
        server.at("/login").get(index!());
        server.at("/register").get(index!());
        server.at("/").get(client_file!("index.html"));
        server.at("/favicon.ico").get(client_file!("favicon.ico"));
        server.at("/").serve_dir(format!("{}/pkg", CLIENT_PATH)).expect("Cannot serve directory");
    }
    fn api() -> tide::Server<()> {
        let mut api = tide::new();
        Self::auth(&mut api);
        Self::serve_table::<PriceSubscription, Schema>(&mut api);
        api.at("/price_history").nest(price_api());
        api
    }
    async fn wss_handler(request: Request<()>) -> tide::Result {
        WebSocket::new(async move |_, ws| {
            websocket::connection(ws).await;
            Ok(())
        })
        .call(request).await
    }
    fn session_middleware() -> tide::sessions::SessionMiddleware<tide::sessions::MemoryStore> {
        tide::sessions::SessionMiddleware::new(
            tide::sessions::MemoryStore::new(),
            session::generate_secret().as_bytes(),
        )
        .with_cookie_name("session")
        .with_session_ttl(Some(std::time::Duration::from_secs(
            session::EXPIRATION_SECS as u64,
        )))
    }
    fn session_validator_middleware() -> impl Middleware<()> {
        tide::utils::Before(async move |mut request: Request<()>| {
            let session = request.session_mut();
            if let Some(expiry) = session.expiry() {
                // time since expiry or (negative) until
                let dt = (Utc::now() - *expiry).num_seconds();
                if dt >= session::STALE_SECS as i64 {
                    // expired and stale
                    session.destroy()
                } else if dt >= 0 {
                    // expired and not stale
                    session.regenerate()
                }
            }
            request
        })
    }
    pub async fn listen(self, addr: SocketAddr) -> std::io::Result<()> {
        self.0.listen(
            TlsListener::build()
                .addrs(addr)
                .cert(keys::to_key_path("tls.crt"))
                .key(keys::to_key_path("tls.key")),
        )
        .await
    }
}

async fn price_history_handler(_: Request<()>) -> tide::Result<Body> {
    match binance()
        .await
        .get_symbol_price_history(PriceHistoryRequest {
            market_pair: "SOLBTC".into(),
            interval: Some(openlimits::model::Interval::OneHour),
            paginator: None,
        })
        .await
    {
        Ok(data) => Ok(Body::from_json(&data)?),
        Err(e) => Err(tide::Error::from_str(500, e.to_string())),
    }
}
fn price_api() -> tide::Server<()> {
    let mut api = tide::new();
    api.at("/").get(price_history_handler);
    api
}

pub async fn run() -> std::io::Result<()> {
    let _telegram_actor = actor_sys_mut()
        .await
        .actor_of::<TelegramActor>("telegram-actor")
        .unwrap();
    let _binance_actor = actor_sys_mut()
        .await
        .actor_of::<BinanceActor>("binance-actor")
        .unwrap();
    TideServer::new()
        .listen(SocketAddr::from(([0, 0, 0, 0], 8000)))
        .await
}
