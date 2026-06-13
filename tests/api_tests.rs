use cdg::api::coingecko::CoinGeckoClient;
use cdg::api::yahoo::YahooClient;
use cdg::cache::Cache;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
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
    let _ = std::fs::remove_file(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"{"gecko_says": "(V3) To the Moon!"}"#;
    let (base_url, _server) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(cache).with_base_url(base_url);
    let val = client.ping().await.unwrap();
    assert_eq!(val["gecko_says"], "(V3) To the Moon!");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_yahoo_fetch_ticker_chart() {
    let db_path = "tests/test_yahoo.db";
    let _ = std::fs::remove_file(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_json = r#"{"chart":{"result":[{"timestamp":[1700000000],"indicators":{"quote":[{"close":[5050.0]}],"adjclose":[{"adjclose":[5050.0]}]}}],"error":null}}"#;
    let (base_url, _server) = start_mock_server(mock_json.to_string()).await;

    let client = YahooClient::new(cache).with_base_url(base_url);
    let json_data = client
        .fetch_ticker_chart("^GSPC", 1700000000, 1700086400)
        .await
        .unwrap();
    assert!(json_data.contains("5050.0"));

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_coingecko_ohlc() {
    let db_path = "tests/test_coingecko_ohlc.db";
    let _ = std::fs::remove_file(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"[[1700000000000, 50000.0, 51000.0, 49000.0, 50500.0]]"#;
    let (base_url, _server) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(cache).with_base_url(base_url);
    let val = client.get_coin_ohlc("bitcoin", "usd", "90").await.unwrap();
    assert_eq!(val.len(), 1);
    assert_eq!(val[0][0], 1700000000000.0);
    assert_eq!(val[0][1], 50000.0);

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_coingecko_tickers() {
    let db_path = "tests/test_coingecko_tickers.db";
    let _ = std::fs::remove_file(db_path);
    let cache = Cache::new(db_path).await.unwrap();

    let mock_response = r#"{"name": "Bitcoin", "tickers": [{"base": "BTC", "target": "USD", "market": {"name": "Binance", "identifier": "binance"}, "last": 60000.0, "volume": 1000.0, "bid_ask_spread_percentage": 0.05}]}"#;
    let (base_url, _server) = start_mock_server(mock_response.to_string()).await;

    let client = CoinGeckoClient::new(cache).with_base_url(base_url);
    let val = client.get_coin_tickers("bitcoin", Some(1)).await.unwrap();
    assert_eq!(val["name"], "Bitcoin");
    assert_eq!(val["tickers"][0]["market"]["name"], "Binance");

    let _ = std::fs::remove_file(db_path);
}
