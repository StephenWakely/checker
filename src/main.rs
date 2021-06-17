use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{debug, error, info};

#[derive(Deserialize, Debug)]
struct Kubernetes {
    container_id: String,
    container_image: String,
    container_name: String,
    pod_labels: HashMap<String, String>,
    pod_name: String,
    pod_namespace: String,
    pod_node_name: String,
}

#[derive(Deserialize, Debug)]
struct Log {
    file: String,
    kubernetes: Kubernetes,
    message: String,
    timestamp: String,
}

impl Log {
    fn count(&self) -> usize {
        let num = self.message.replace("COUNT ", "");
        match num.parse() {
            Ok(num) => num,
            Err(_) => {
                error!("Cant parse {}", self.message);
                0
            }
        }
    }
}

async fn health(_req: HttpRequest) -> impl Responder {
    debug!("health check");
    "groovy"
}

async fn log(data: web::Data<Data>, log: String) -> impl Responder {
    debug!("log {:?}", log);
    let logs: Result<Vec<Log>, _> = serde_json::from_str(&log);
    match logs {
        Ok(logs) => {
            for log in logs {
                if log.kubernetes.container_name == "zork" && log.message.contains("COUNT ") {
                    let mut count = data.counter.lock().unwrap();
                    let last = count.get(&log.kubernetes.pod_name).cloned().unwrap_or(0);

                    if last + 1 != log.count() {
                        error!("oops {} {}", log.kubernetes.pod_name, log.count());
                    }

                    count.insert(log.kubernetes.pod_name.clone(), log.count());
                }
            }
        }
        Err(_) => (),
    }

    HttpResponse::Ok().finish()
}

async fn info(data: web::Data<Data>) -> impl Responder {
    let count = data.counter.lock().unwrap();
    debug!("info {:?}", count);
    web::Json(count.clone())
}

struct Data {
    counter: Arc<Mutex<HashMap<String, usize>>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Listening on 8000");

    let data = Arc::new(Mutex::new(HashMap::new()));
    HttpServer::new(move || {
        App::new()
            .data(Data {
                counter: data.clone(),
            })
            .route("/health", web::get().to(health))
            .route("/log", web::post().to(log))
            .route("/info", web::get().to(info))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
