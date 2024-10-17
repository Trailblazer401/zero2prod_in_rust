use actix_web::{HttpRequest, Responder};

pub async fn greet(req: HttpRequest) -> impl Responder {    // greet返回一个实现了Responder trait的（任何）类型
    let name = req.match_info().get("name").unwrap_or("World");    // unwarp_or 从 Option 或 Result中提取值，返回 Some（Option）/ Ok（Result）中的值或 default （若结果为 None / Err），但不进行错误处理
    format!("Hello {}!", &name)
}