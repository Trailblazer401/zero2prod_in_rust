//! src/startup.rs

use std::net::TcpListener;
use actix_web::{
    HttpServer, 
    App, 
    web, 
    dev::Server
};
use crate::{configurations::{DatabaseSettings, Settings}, routes::{health_check, subscribe}};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let sender_email = configuration.email_client.sender().expect("Invalid sender email");
        
        let timeout = configuration.email_client.timeout();
        
        let email_client = EmailClient::new(
            configuration.email_client.base_url, 
            sender_email, 
            configuration.email_client.authorization_token, 
            timeout
        );
        
        let connection_pool = get_connection_pool(&configuration.database);
        
        let addr = format!(
            "{}:{}", 
            configuration.application.host, 
            configuration.application.port
        );
        
        let listener = TcpListener::bind(addr)?;

        let port = listener.local_addr().unwrap().port();
        
        let server = run(
            listener, 
            connection_pool, 
            email_client
        )?;

        Ok(Self {port, server})
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(listener: TcpListener, db_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);    // 此处使用智能指针（计数指针Arc）包装connection，这使得原本不具有clone trait的PgPool（PgConnection）类型通过Arc计数指针实现可克隆性质，每次克隆使得Arc计数+1
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {    // 使用 move 将外部变量（此处是 db_pool）的所有权转移到闭包内部，以期能在闭包内部安全地调用db_pool的clone方法
        App::new()    // 使用闭包而不是直接使用App类型作为HttpServer::new方法的参数，为每一个http连接调用一次闭包实例化一个App对象，使得来自不同客户端的连接实现隔离
            // .wrap(Logger::default())
            .wrap(TracingLogger::default())
            // .route("/", web::get().to(greet))
            // .route("/{name}", web::get().to(greet))    // {name} 是一占位符，在客户端访问某URL路径时（如“/John”）匹配路径中的实际值。通过制定路由route("/{name}")，actix web在处理对应路由请求时会自动从路径中提取name参数，并通过web::get().to(greet)将对该路由的http get请求映射到处理函数greet实现参数传递，该参数传递由actix web通过函数签名自动推断完成，因此greet也没有显式的参数列表
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())    // 使用app_data方法将PgPool(PgConnection)连接对象注册为该App实例的一部分，这里使用Arc实现clone trait，以使连接对每一个App实例可克隆
            .app_data(email_client.clone())
    })    // 此处 HttpServer::new(|| {...}) 中使用闭包进行参数传递，|...| 表示闭包的参数列表，该处没有传入闭包的参数，故参数列表为空（ || )，{...}表示闭包的实现体，包含闭包的执行逻辑，该闭包返回一个配置了路由的App实例
    .listen(listener)?    // 此处 ? 运算的对象是由bind函数运行返回的 Result<Self> 即 Result<HttpServer, E>，绑定成功则 Result<Self> 会是 Ok(HttpServer)，则该链式调用继续执行run方法；绑定失败则是 Err(std::io::Error)，则整个run函数在此处停止并返回该Err类型
    .run();
    // .await
    Ok(server)
}