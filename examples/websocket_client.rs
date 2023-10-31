use futures::stream::FuturesUnordered;
use futures::{SinkExt, StreamExt};
use std::borrow::Cow;
use std::ops::ControlFlow;
use std::time::Instant;

// we will use tungstenite for websocket client impl (same library as what axum is using)
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::{frame::coding::CloseCode, CloseFrame, Message},
};

use anyhow::{anyhow as error, Result};

const N_CLIENTS: usize = 2; //set to desired number
const SERVER: &str = "ws://127.0.0.1:3000/ws";

#[derive(Debug, Clone, serde::Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct LoginResponse {
    token: String,
}

async fn fetch_token(host: &str) -> Result<String> {
    let login_endpoint = format!("http://{host}/user/users/login");

    let client = reqwest::Client::new();
    let body = LoginRequest {
        email: String::from("harris1.perceptron@gmail.com"),
        password: String::from("meow12345"),
    };

    let login_response = client.post(login_endpoint).json(&body).send().await?;
    let response: LoginResponse = login_response.json().await?;
    let token = response.token.to_string();
    Ok(token)
}

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    let host = std::env::var("HOST_ADDRESS").unwrap_or(String::from("127.0.0.1:3000"));

    let websocket_address = format!("ws://{host}/ws");

    let token = fetch_token(&host).await?;

    log::info!("Got login response: {}", token);

    let start_time = Instant::now();
    //spawn several clients that will concurrently talk to the server
    let mut clients = (0..N_CLIENTS)
        .map(|cli| tokio::spawn(spawn_client(cli)))
        .collect::<FuturesUnordered<_>>();

    //wait for all our clients to exit
    while clients.next().await.is_some() {}

    let end_time = Instant::now();

    //total time should be the same no matter how many clients we spawn
    println!(
        "Total time taken {:#?} with {N_CLIENTS} concurrent clients, should be about 6.45 seconds.",
        end_time - start_time
    );

    Ok(())
}

//creates a client. quietly exits on failure.
async fn spawn_client(who: usize) {
    let token = fetch_token("127.0.0.1:3000").await.unwrap();
    let bearer_token = format!("Bearer {token}");
    let request = http::Request::builder()
        .uri(SERVER)
        .header("sec-websocket-key", "foo")
        .header("Authorization", bearer_token)
        .header("host", "localhost:3000")
        .header("upgrade", "websocket")
        .header("connection", "upgrade")
        .header("sec-websocket-version", 13)
        .body(())
        .unwrap();

    let ws_stream = match connect_async(request).await {
        Ok((stream, response)) => {
            println!("Handshake for client {who} has been completed");
            // This will be the HTTP response, same as with server this is the last moment we
            // can still access HTTP stuff.
            println!("Server response was {response:?}");
            stream
        }
        Err(e) => {
            println!("WebSocket handshake for client {who} failed with {e}!");
            return;
        }
    };

    let (mut sender, mut receiver) = ws_stream.split();

    //we can ping the server for start
    sender
        .send(Message::Ping("Hello, Server!".into()))
        .await
        .expect("Can not send!");

    //spawn an async sender to push some more messages into the server
    let mut send_task = tokio::spawn(async move {
        for i in 1..30 {
            // In any websocket error, break loop.
            if sender
                .send(Message::Text(format!("Message number {i}...")))
                .await
                .is_err()
            {
                //just as with server, if send fails there is nothing we can do but exit.
                return;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        // When we are done we may want our client to close connection cleanly.
        println!("Sending close to {who}...");
        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Cow::from("Goodbye"),
            })))
            .await
        {
            println!("Could not send Close due to {e:?}, probably it is ok?");
        };
    });

    //receiver just prints whatever it gets
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // print message and break if instructed to do so
            if process_message(msg, who).is_break() {
                break;
            }
        }
    });

    //wait for either task to finish and kill the other task
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }
}

/// Function to handle messages we get (with a slight twist that Frame variant is visible
/// since we are working with the underlying tungstenite library directly without axum here).
fn process_message(msg: Message, who: usize) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} got str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {} got {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} got close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow got close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} got pong with {v:?}");
        }
        // Just as with axum server, the underlying tungstenite websocket library
        // will handle Ping for you automagically by replying with Pong and copying the
        // v according to spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} got ping with {v:?}");
        }

        Message::Frame(_) => {
            unreachable!("This is never supposed to happen")
        }
    }
    ControlFlow::Continue(())
}
