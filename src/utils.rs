//! Utilities for both clients and servers.

use std::error::Error;

pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;

use async_std::prelude::*;
use serde::Serialize;
use std::marker::Unpin;

pub async fn send_as_json<S, P>(outbound: &mut S, packet: &P) -> ChatResult<()>
where
    S: async_std::io::Write + Unpin,
    P: Serialize,
{
    let mut json = serde_json::to_string(&packet)?;
    json.push('\n');
    outbound.write_all(json.as_bytes()).await?;         // outbound对应接收端地址
    Ok(())
}

use serde::de::DeserializeOwned;

pub fn receive_as_json<S, P>(inbound: S) -> impl Stream<Item = ChatResult<P>>
    where S: async_std::io::BufRead + Unpin,
          P: DeserializeOwned,
    // pub trait DeserializeOwned
    // where
    //     Self: for<'de> Deserialize<'de>,
    // 表示：一个类型 Self 要想成为 DeserializeOwned，它必须满足一个条件：
    // 对于任何（for all）可能的生命周期 'de，Self 都必须实现 Deserialize<'de>
    // 如果Self包含了引用，例如，Struct UserView<'a> { name: &'a str }，
    // 则'a必须与某个具体的生命周期绑定，从而不能是任意的
    // for<'de>实际上表示的是任意生命周期的意思，因而只有具备控制权的类型才满足要求
{
    // 要求S必须是Unpin的，是因为inbound.lines()返回一个异步的Stream，
    // 该Stream内部持有对inbound的引用，因此为了保证S可以安全的移动，需要限制为Unpin
    // 实际上，S的限制中可以将Unpin删除，并将inbound改为Box::pin(inbound)
    // 但是，这样做没有必要，因为Unpin在Rust中才是常见的情形，使用Pin进行包装会增加复杂性和内存开销
    // 并且使使用更加复杂
    inbound.lines()         // inbound对应发送端地址
        .map(|line_result| -> ChatResult<P> {
            let line = line_result?;
            let parsed = serde_json::from_str::<P>(&line)?;
            Ok(parsed)
        })
}
