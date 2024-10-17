//! lib.rs

// use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
// use std::net::TcpListener;

// #[derive(serde::Deserialize)]    // 该处的属性宏#[derive()]用于自动为 FormData 结构体实现来自serde库的 trait: serde::Deserialize
// struct FormData {
//     email: String,
//     name: String
// }

// async fn _greet(req: HttpRequest) -> impl Responder {    // greet返回一个实现了Responder trait的（任何）类型
//     let name = req.match_info().get("name").unwrap_or("World");    // unwarp_or 从 Option 或 Result中提取值，返回 Some（Option）/ Ok（Result）中的值或 default （若结果为 None / Err），但不进行错误处理
//     format!("Hello {}!", &name)
// }

// async fn health_check() -> HttpResponse {
//     HttpResponse::Ok().finish()
// }

// async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
//     HttpResponse::Ok().finish()
// }

// #[tokio::main]    // 通过 tokio 宏使 main 函数在 tokio 提供的异步运行时上运行，由 tokio 宏负责补充样本代码
// pub async fn run() -> std::io::Result<()> {    // 此处 Result<()> 中的（）为 Rust 中的 unit 类型，表示若 run 函数运行成功则返回一个Ok(())，仅通知运行成功但不返回有意义的数据
// pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
//     let server = HttpServer::new(|| {
//         App::new()
//             // .route("/", web::get().to(greet))
//             // .route("/{name}", web::get().to(greet))    // {name} 是一占位符，在客户端访问某URL路径时（如“/John”）匹配路径中的实际值。通过制定路由route("/{name}")，actix web在处理对应路由请求时会自动从路径中提取name参数，并通过web::get().to(greet)将对该路由的http get请求映射到处理函数greet实现参数传递，该参数传递由actix web通过函数签名自动推断完成，因此greet也没有显式的参数列表
//             .route("/health_check", web::get().to(health_check))
//             .route("/subscriptions", web::post().to(subscribe))
//     })    // 此处 HttpServer::new(|| {...}) 中使用闭包进行参数传递，|...| 表示闭包的参数列表，该处没有传入闭包的参数，故参数列表为空（ || )，{...}表示闭包的实现体，包含闭包的执行逻辑，该闭包返回一个配置了路由的App实例
//     .listen(listener)?    // 此处 ? 运算的对象是由bind函数运行返回的 Result<Self> 即 Result<HttpServer, E>，绑定成功则 Result<Self> 会是 Ok(HttpServer)，则该链式调用继续执行run方法；绑定失败则是 Err(std::io::Error)，则整个run函数在此处停止并返回该Err类型
//     .run();
//     // .await
//     Ok(server)
// }

pub mod configurations;
pub mod routes;
pub mod startup;