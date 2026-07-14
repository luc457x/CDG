use cdg::api::coingecko::CoinGeckoClient;
use cdg::api::yahoo::YahooClient;
use cdg::cache::Cache;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn cleanup_db(db_path: &str) {
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(format!("{}-shm", db_path));
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
}

async fn start_mock_server(response_body: String) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();
    let base_url = format!("http://127.0.0.1:{}", port);

    let handle = tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            let mut buf = [0; 1024];
            let _ = socket.read(&mut buf).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });

    (base_url, handle)
}

#[tokio::test]
async fn test_coingecko_ping() {
    let db_path = "tests/test_coingecko_ping.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"{"gecko_says": "(V3) To the Moon!"}"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);
    let val = client.ping().await.unwrap();
    assert_eq!(val["gecko_says"], "(V3) To the Moon!");

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_yahoo_fetch_ticker_chart() {
    let db_path = "tests/test_yahoo.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_json = r#"{"chart":{"result":[{"timestamp":[1700000000],"indicators":{"quote":[{"close":[5050.0]}],"adjclose":[{"adjclose":[5050.0]}]}}],"error":null}}"#;
    let (base_url, _server_handle) = start_mock_server(mock_json.to_string()).await;

    let client = YahooClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);
    let json_data = client
        .fetch_ticker_chart("^GSPC", 1700000000, 1700086400)
        .await
        .unwrap();
    assert!(json_data.contains("5050.0"));

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_coingecko_ohlc() {
    let db_path = "tests/test_coingecko_ohlc.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"[[1700000000000, 50000.0, 51000.0, 49000.0, 50500.0]]"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);
    let val = client.get_coin_ohlc("bitcoin", "usd", "90").await.unwrap();
    assert_eq!(val.len(), 1);
    assert_eq!(val[0][0], 1700000000000.0);
    assert_eq!(val[0][1], 50000.0);

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_coingecko_tickers() {
    let db_path = "tests/test_coingecko_tickers.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"{"name": "Bitcoin", "tickers": [{"base": "BTC", "target": "USD", "market": {"name": "Binance", "identifier": "binance"}, "last": 60000.0, "volume": 1000.0, "bid_ask_spread_percentage": 0.05}]}"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);
    let val = client.get_coin_tickers("bitcoin", Some(1)).await.unwrap();
    assert_eq!(val["name"], "Bitcoin");
    assert_eq!(val["tickers"][0]["market"]["name"], "Binance");

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_check_coin_exact() {
    use cdg::api::coingecko::CoinResolution;

    let db_path = "tests/test_check_coin_exact.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"[
        {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin"}
    ]"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);

    assert_eq!(
        client.check_coin_id("Bitcoin").await.unwrap(),
        CoinResolution::Exact("bitcoin".to_string())
    );

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_check_coin_ambiguous() {
    use cdg::api::coingecko::CoinResolution;

    let db_path = "tests/test_check_coin_ambiguous.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"[
        {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin"},
        {"id": "bitcoin-cash", "symbol": "bch", "name": "Bitcoin Cash"}
    ]"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);

    let mut resolved = match client.check_coin_id("bit").await.unwrap() {
        CoinResolution::Ambiguous(list) => list,
        other => panic!("Expected Ambiguous, got {:?}", other),
    };
    resolved.sort();
    let mut expected = vec!["bitcoin-cash".to_string(), "bitcoin".to_string()];
    expected.sort();
    assert_eq!(resolved, expected);

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_check_coin_not_found() {
    use cdg::api::coingecko::CoinResolution;

    let db_path = "tests/test_check_coin_not_found.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"[
        {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin"}
    ]"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);

    assert_eq!(
        client.check_coin_id("unknown_token").await.unwrap(),
        CoinResolution::NotFound
    );

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_coingecko_list_coins() {
    let db_path = "tests/test_coingecko_list_coins.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"[{"id": "bitcoin", "symbol": "btc", "name": "Bitcoin"}]"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);
    let val = client.get_coins_list().await.unwrap();
    assert_eq!(val.len(), 1);
    assert_eq!(val[0]["id"], "bitcoin");

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_coingecko_trending() {
    let db_path = "tests/test_coingecko_trending.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"{"coins": [{"item": {"id": "bitcoin", "name": "Bitcoin"}}]}"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);
    let val = client.get_search_trending().await.unwrap();
    assert_eq!(val["coins"][0]["item"]["id"], "bitcoin");

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_yahoo_ping() {
    let db_path = "tests/test_yahoo_ping.db";
    cleanup_db(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_json = r#"{"chart":{"result":[{"timestamp":[1700000000],"indicators":{"quote":[{"close":[5050.0]}],"adjclose":[{"adjclose":[5050.0]}]}}],"error":null}}"#;
    let (base_url, _server_handle) = start_mock_server(mock_json.to_string()).await;

    let client = YahooClient::new(std::sync::Arc::new(cache))
        .unwrap()
        .with_base_url(base_url);
    let val = client.ping().await;
    assert!(val.is_ok());

    cleanup_db(db_path);
    drop(_server_handle);
}

#[tokio::test]
async fn test_clients_with_progress_bar() {
    let db_path = "tests/test_clients_pb.db";
    cleanup_db(db_path);
    let cache = std::sync::Arc::new(Cache::new(db_path).await.unwrap());

    let pb = indicatif::ProgressBar::hidden();

    let _cg_client = CoinGeckoClient::new(cache.clone())
        .unwrap()
        .with_progress_bar(pb.clone());

    let _yahoo_client = YahooClient::new(cache.clone())
        .unwrap()
        .with_progress_bar(pb);

    cleanup_db(db_path);
}

#[tokio::test]
async fn test_coingecko_market_chart_range_same_day_alignment() {
    let db_path = "tests/test_coingecko_cache_rounding.db";
    cleanup_db(db_path);
    let cache = std::sync::Arc::new(Cache::new(db_path).await.unwrap());

    let mock_response = r#"{"prices": [[1700000000000, 50000.0]], "market_caps": [], "total_volumes": []}"#;
    let (base_url, _server_handle) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(cache)
        .unwrap()
        .with_base_url(base_url)
        .with_ttl(300);

    let now_1 = 1719878400 + 3600; 
    let now_2 = 1719878400 + 72000; 

    let rounded_now_1 = (now_1 / 86400) * 86400;
    let rounded_now_2 = (now_2 / 86400) * 86400;

    assert_eq!(rounded_now_1, rounded_now_2);

    let from_1 = rounded_now_1 - 10 * 86400;
    let to_1 = rounded_now_1;

    let from_2 = rounded_now_2 - 10 * 86400;
    let to_2 = rounded_now_2;

    let res_1 = client.get_coin_market_chart_range("bitcoin", "usd", from_1, to_1).await.unwrap();
    assert_eq!(res_1["prices"][0][0], 1700000000000.0);

    let res_2 = client.get_coin_market_chart_range("bitcoin", "usd", from_2, to_2).await.unwrap();
    assert_eq!(res_2["prices"][0][0], 1700000000000.0);

    cleanup_db(db_path);
    drop(_server_handle);
}

