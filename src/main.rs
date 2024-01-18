mod hot_reload;

use std::convert::Infallible;
use std::rc::Rc;
use std::sync::Arc;
use std::task::Poll;
use axum::extract::{Path, State};
use axum::response::{Html, Sse};
use axum::response::sse::{Event, KeepAlive};
use axum::routing::get;
use axum::Router;
use futures::stream::{self, Stream};
use futures_util::{FutureExt};
use tera::Tera;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, RwLock};
use tower_http::services::ServeDir;
use tokio_stream::StreamExt as _ ;


#[cfg(not(debug_assertions))]
fn example() {
    println!("Debugging disabled");
}

//#[cfg(debug_assertions)]

async fn handler_page(state: State<AppState>, Path(file_name): Path<String>) -> Html<String> {
    handler(file_name, state).await
}

async fn handler_index(state: State<AppState>) -> Html<String> {
    handler("index".to_string(), state).await
}

async fn handler(
    page: String,
    State(mut state): State<AppState>
) -> Html<String> {
    let tera = state.tera.read().await;

    if !tera.templates.contains_key(&format!("{}.html", page)) {
        return Html("404".to_string());
    }

    let ctx = tera::Context::new();
    let html = tera.render(&format!("{}.html", page), &ctx).unwrap();

    Html(html)
}


#[cfg(debug_assertions)]
async fn hot_reload_handler(State(mut state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::once(async move {
        state.hot_reload.recv().await.unwrap();

        Ok(Event::default().data("reload"))
    });


    Sse::new(stream).keep_alive(KeepAlive::default())


}

struct AppState {
    tera: Arc<RwLock<Tera>>,
    #[cfg(debug_assertions)]
    hot_reload: Receiver<bool>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            tera: self.tera.clone(),
            #[cfg(debug_assertions)]
            hot_reload: self.hot_reload.resubscribe()
        }
    }
}

fn nest() -> Router<AppState>
{
    #[cfg(debug_assertions)]
    {
        return Router::new()
            .route("/__hot_reload", get(hot_reload_handler))

    }

    Router::new()
}

#[tokio::main]
async fn main() {
    let (sender, receiver ) = broadcast::channel::<bool>(10);
    let tera = Tera::new("src/view/**/*").unwrap();

    let rwlock = Arc::new(RwLock::new(tera));

    #[cfg(debug_assertions)]
    hot_reload::watcher::watch(sender, rwlock.clone());

    let state = AppState {
        tera: rwlock,
        #[cfg(debug_assertions)]
        hot_reload: receiver,
    };

    let app = Router::new()
        .nest_service("/dist", ServeDir::new("./dist"))
        .route("/", get(handler_index))
        .route("/:file_name", get(handler_page))
        .merge( nest())
        .with_state(state);



    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    println!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
