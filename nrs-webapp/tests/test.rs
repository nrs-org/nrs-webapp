#[tokio::test]
async fn test_test() -> anyhow::Result<()> {
    let hc = httpc_test::new_client("http://localhost:3621")?;

    hc.do_get("/test").await?.print().await?;
    Ok(())
}
