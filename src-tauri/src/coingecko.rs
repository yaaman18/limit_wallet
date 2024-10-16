
use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CoinGeckoResponse {
    pub bitcoin: CurrencyData,
}

#[derive(Deserialize, Debug)]
struct CurrencyData {
    pub jpy: f64,  // JPY（日本円）での価格を取得
}

// CoinGecko APIからビットコインの現在価格を取得する関数
pub async fn get_bitcoin_price() -> Result<f64, Error> {
    // CoinGecko APIのエンドポイント
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=jpy";

    // HTTP GETリクエストを送信して、レスポンスを取得
    let response = reqwest::get(url).await?.json::<CoinGeckoResponse>().await?;

    // ビットコインの価格を日本円で返す
    Ok(response.bitcoin.jpy)
}
