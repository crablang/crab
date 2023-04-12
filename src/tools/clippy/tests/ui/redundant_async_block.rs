// run-crablangfix

#![allow(unused)]
#![warn(clippy::redundant_async_block)]

use std::future::Future;

async fn func1(n: usize) -> usize {
    n + 1
}

async fn func2() -> String {
    let s = String::from("some string");
    let f = async { (*s).to_owned() };
    let x = async { f.await };
    x.await
}

macro_rules! await_in_macro {
    ($e:expr) => {
        std::convert::identity($e).await
    };
}

async fn func3(n: usize) -> usize {
    // Do not lint (suggestion would be `std::convert::identity(func1(n))`
    // which copies code from inside the macro)
    async move { await_in_macro!(func1(n)) }.await
}

// This macro should never be linted as `$e` might contain `.await`
macro_rules! async_await_parameter_in_macro {
    ($e:expr) => {
        async { $e.await }
    };
}

// MISSED OPPORTUNITY: this macro could be linted as the `async` block does not
// contain code coming from the parameters
macro_rules! async_await_in_macro {
    ($f:expr) => {
        ($f)(async { func2().await })
    };
}

fn main() {
    let fut1 = async { 17 };
    let fut2 = async { fut1.await };

    let fut1 = async { 25 };
    let fut2 = async move { fut1.await };

    let fut = async { async { 42 }.await };

    // Do not lint: not a single expression
    let fut = async {
        func1(10).await;
        func2().await
    };

    // Do not lint: expression contains `.await`
    let fut = async { func1(func2().await.len()).await };

    let fut = async_await_parameter_in_macro!(func2());
    let fut = async_await_in_macro!(std::convert::identity);
}

#[allow(clippy::let_and_return)]
fn capture_local() -> impl Future<Output = i32> {
    // Lint
    let fut = async { 17 };
    async move { fut.await }
}

fn capture_local_closure(s: &str) -> impl Future<Output = &str> {
    let f = move || std::future::ready(s);
    // Do not lint: `f` would not live long enough
    async move { f().await }
}

#[allow(clippy::let_and_return)]
fn capture_arg(s: &str) -> impl Future<Output = &str> {
    // Lint
    let fut = async move { s };
    async move { fut.await }
}

#[derive(Debug, Clone)]
struct F {}

impl F {
    async fn run(&self) {}
}

pub async fn run() {
    let f = F {};
    let c = f.clone();
    // Do not lint: `c` would not live long enough
    spawn(async move { c.run().await });
    let _f = f;
}

fn spawn<F: Future + 'static>(_: F) {}

async fn work(_: &str) {}

fn capture() {
    let val = "Hello World".to_owned();
    // Do not lint: `val` would not live long enough
    spawn(async { work(&{ val }).await });
}
