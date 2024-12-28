use std::collections::BTreeMap;
use std::io;
use std::sync::Arc;

use axum;
use base64;
use base64::Engine;
use ring;
use serde;
use tokio;
use tokio::sync::RwLock as TokioRwLock;

struct Session {
    user: Option<String>,
    description: String,
    authenticated: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SessionId {
    id: [u8; 16],
}

impl TryFrom<&str> for SessionId {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, ()> {
        let mut id: [u8; 16] = [0; 16];
        match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode_slice(value, &mut id) {
            Ok(_) => Ok(Self { id }),
            Err(_) => Err(()),
        }
    }
}

impl From<&SessionId> for String {
    fn from(value: &SessionId) -> Self {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value.id)
    }
}

struct AppState {
    sessions: TokioRwLock<BTreeMap<SessionId, Arc<TokioRwLock<Session>>>>,
    rng: TokioRwLock<ring::rand::SystemRandom>,
}

impl AppState {
    fn new() -> Self {
        Self {
            sessions: TokioRwLock::new(BTreeMap::new()),
            rng: TokioRwLock::new(ring::rand::SystemRandom::new()),
        }
    }
}

#[derive(serde::Serialize)]
struct NewSessionResponse {
    id_base64: String,
}

async fn new_session(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::response::Json<NewSessionResponse> {
    let session_id = SessionId {
        id: ring::rand::generate(&(*state.rng.read().await))
            .unwrap()
            .expose(),
    };
    let session = Arc::new(TokioRwLock::new(Session {
        user: None,
        description: String::from("Some session..."),
        authenticated: false,
    }));

    {
        let mut sessions_locked = state.sessions.write().await;
        sessions_locked.insert(session_id.clone(), session);
    }

    println!("Created new session {}", String::from(&session_id));

    axum::response::Json(NewSessionResponse {
        id_base64: (&session_id).into(),
    })
}

#[derive(serde::Deserialize)]
struct AuthenticateForm {
    session_id: String,
    password: String,
}

async fn authenticate(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<AuthenticateForm>,
) -> axum::response::Response {
    let session_id: Result<SessionId, ()> = form.session_id.as_str().try_into();
    if let Err(_) = session_id {
        return axum::response::Response::builder()
            .status(400)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::new(String::from(
                "{\"error\":\"malformed session id\"}",
            )))
            .unwrap();
    }
    let session_id = session_id.unwrap();
    let session = {
        let sessions_locked = state.sessions.read().await;
        sessions_locked
            .get(&session_id)
            .and_then(|x| Some(x.clone()))
    };
    match session {
        Some(session) => axum::response::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::new(format!(
                "{{\"success\":\"session {} authenticated succesfully\"}}",
                form.session_id
            )))
            .unwrap(),
        None => axum::response::Response::builder()
            .status(400)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::new(format!(
                "{{\"error\":\"session {} doesn't exist\"}}",
                form.session_id
            )))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Hello, world!");

    let app_state = Arc::new(AppState::new());
    let app = axum::Router::new()
        .route(
            "/api/new_session",
            axum::routing::post(new_session).with_state(app_state.clone()),
        )
        .route(
            "/api/authenticate",
            axum::routing::post(authenticate).with_state(app_state.clone()),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
