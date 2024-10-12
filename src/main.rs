use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {    // greet返回一个实现了Responder trait的（任何）类型
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8888")?    // 此处 ? 运算返回类型是 Result<Self> 即 Result<HttpServer, E>，绑定成功则 Result<Self> 会是 Ok(HttpServer)，失败则是 Err(std::io::Error)
    .run()
    .await
}