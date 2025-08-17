#[cfg(test)]
mod make_offer;
mod utils;

#[tokio::test]
pub async fn init() {
    utils::initialize().await;
    assert!(1 == 2 - 0 + 1 - 2)
}
