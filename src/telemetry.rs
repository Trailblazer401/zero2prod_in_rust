//! src/telemetry.rs

use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::MakeWriter, 
    layer::SubscriberExt, 
    EnvFilter, 
    Registry
};

pub fn get_subscriber<Sink>(    // <> 表示该函数使用泛型参数（本函数中占位符为 Sink）
    name: String, 
    env_filter: String,
    sink: Sink
) -> impl Subscriber + Sync + Send
    where 
        Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
        // for<'a> trait<'a> 表示泛型生命周期绑定（或称高阶特质约束 HRTB），具体指 Sink 泛型必须实现 MakeWriter trait，且该 trait 可作用于*任意*生命周期（由for<‘a>指定）
        // ‘static 表示 Sink 泛型的所有数据在程序运行期间都必须有效（静态生命周期），即 Sink 只能持有拥有 ’static 生命周期的引用或不持有任何引用
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));    // unwrap_or_else  match Some(T) => T, None => op 
    let formatting_layer = BunyanFormattingLayer::new(
        name, 
        sink
    );

    Registry::default()
        .with(env_filter)    // EnvFilter Layer 用于筛选 span 数据
        .with(JsonStorageLayer)    // JsonStorageLayer 用于处理 span 数据为 Json格式
        .with(formatting_layer)    // BunyanFormattingLayer 以 bunyan 兼容格式输出 Json 数据
    // 激活订阅器 Registry 用于收集各 Layer 周到的数据以及跟踪 span
}

pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");    // 指定处理 span 的订阅器
}