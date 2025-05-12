mod temper;

use std::env;
use std::process;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use axum::extract;
use axum::{Router, routing::get};

const UPDATE_INTERVAL: Duration = Duration::from_secs(3);

const HEADER: &str = "# HELP temper_temp The temperature measured by a TEMPer USB stick
# TYPE temper_temp gauge\n";

#[derive(Clone)]
struct State {
    temp: Arc<RwLock<Vec<f32>>>,
}

#[tokio::main]
async fn main() {
    let temper_instance = temper::Temper::new().unwrap();

    let mut sticks = temper_instance.get_sticks();

    for stick in &mut sticks {
        if let Err(e) = stick.init() {
            eprintln!("Error: {e}");
            process::exit(1);
        } else {
            println!("{}", stick.get_temp().unwrap());
        }
    }

    let state = State {
        temp: Arc::new(RwLock::new(Vec::with_capacity(sticks.len()))),
    };

    state
        .temp
        .clone()
        .write()
        .unwrap()
        .resize(sticks.len(), 0.0f32);

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(state.clone());

    tokio::spawn(async move {
        let p = state.temp.clone();
        loop {
            {
                let mut temp = p.write().unwrap();
                for i in 0..sticks.len() {
                    temp[i] = sticks[i].get_temp().unwrap();
                }
            }
            tokio::time::sleep(UPDATE_INTERVAL).await;
        }
    });

    let listen_addr = format!("0.0.0.0:{}", {
        if env::args().len() > 1 {
            env::args().nth(1).unwrap()
        } else {
            String::from("9177")
        }
    });

    let listener = tokio::net::TcpListener::bind(listen_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn metrics_handler(extract::State(state): extract::State<State>) -> String {
    let mut msg = String::from(HEADER);

    {
        let temps = state.temp.read().unwrap();
        let temps_len = temps.len();

        for i in 0..temps_len {
            msg.push_str(&format!(
                "temper_temp{{stick=\"{}\"}} {}\n",
                temps_len - i - 1,
                temps[i]
            ));
        }
    }

    msg
}
