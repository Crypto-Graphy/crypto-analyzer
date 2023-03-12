pub mod coin_gecko {
    extern crate reqwest;
    extern crate rust_decimal;

    use std::collections::HashMap;

    use self::rust_decimal::Decimal;

    // use rust_decimal::Decimal;

    pub const BASE_API_URL: &str = "https://api.coingecko.com/api/v3";
    pub const PRICE_ROUTE: &str = "/simple/price";
    pub const VS_CURRENCY: &str = "usd";

    /// Converts exchange ticker into CoinGecko id.
    /// ```
    /// use coin_gecko::coin_gecko::ticker_to_id;
    /// let currency = "btc";
    /// let id = ticker_to_id(currency).unwrap(); // unwrap shouldn't be used outside this test since this could return None.
    /// assert_eq!(id, "bitcoin".to_string());
    /// ```
    pub fn ticker_to_id(currency: &str) -> Option<String> {
        let currency = currency.to_ascii_lowercase();
        let currency_ref = currency.as_str();

        match currency_ref {
            "eth" | "eth2" | "eth2.s" => Some("ethereum".to_string()),
            "cgld" => Some("celo".to_string()),
            "btc" => Some("bitcoin".to_string()),
            "algo" | "algo.s" => Some("algorand".to_string()),
            "near" => Some("near".to_string()),
            "amp" => Some("amp-token".to_string()),
            "icp" => Some("internet-computer".to_string()),
            "fil" => Some("filecoin".to_string()),
            "comp" => Some("compound-coin".to_string()),
            "fet" => Some("fetch-ai".to_string()),
            "ada" | "ada.s" => Some("cardano".to_string()),
            "gtc" => Some("gitcoin".to_string()),
            "dnt" => Some("nucypher".to_string()),
            "sol" | "sol.s" => Some("solana".to_string()),
            "usdt" => Some("tether".to_string()),
            "gal" => Some("gallant".to_string()),
            "dai" => Some("dai".to_string()),
            "dot" | "dot.s" => Some("polkadot".to_string()),
            "usdc" => Some("usd-coin".to_string()),
            "storj" => Some("storj".to_string()),
            "xtz" => Some("tezos".to_string()),
            "skl" => Some("skale".to_string()),
            "ankr" => Some("ankr".to_string()),
            "forth" => Some("ampleforth-governance-token".to_string()),
            "matic" => Some("matic-network".to_string()),
            "grt" => Some("the-graph".to_string()),
            "xcn" => Some("chain-2".to_string()),
            "xlm" => Some("stellar".to_string()),
            "atom" | "atom.s" => Some("cosmos".to_string()),
            "pols" => Some("polkastarter".to_string()),
            "scrt" | "scrt.s" => Some("secret".to_string()),
            _ => {
                println!("{currency_ref} not matched");
                None
            }
        }
    }

    pub fn get_current_price(coin_gecko_ids: String) -> Result<HashMap<String, Decimal>, String> {
        let queries = format!("?ids={}&vs_currencies={}", coin_gecko_ids, VS_CURRENCY);
        let url = format!("{}{}", BASE_API_URL, PRICE_ROUTE);
        let url = format!("{}{}", url, queries);

        println!("Getting price with url: {}", &url);

        match reqwest::blocking::get(url) {
            Ok(response) => match response.status().is_success() {
                true => Ok(
                    match response.json::<HashMap<String, HashMap<String, Decimal>>>() {
                        Ok(price_data) => price_data
                            .into_iter()
                            .filter_map(|(symbol, mut price_map)| {
                                price_map.remove("usd").map(|price| (symbol, price))
                            })
                            .collect::<HashMap<String, Decimal>>(),
                        Err(e) => return Err(format!("Error attempting to convert http price request to object, original error: {}", e)),
                    },
                ),
                false => Err(format!(
                    "Pricing data response was not successful: {}",
                    response.text().unwrap_or_default()
                )),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod ticker_to_id {
    use crate::coin_gecko::ticker_to_id;

    #[test]
    fn converts_tickers_to_ids() {
        let tickers = [
            "eth", "eth2", "btc", "algo", "near", "amp", "icp", "fil", "comp", "fet", "ada", "gtc",
            "dnt", "sol", "usdt", "gal", "dai", "dot", "usdc", "storj", "xtz", "skl", "ankr",
            "forth", "matic", "grt", "xcn", "xlm", "atom", "pols",
        ];

        let expected = vec![
            "ethereum".to_string(),
            "ethereum".to_string(),
            "bitcoin".to_string(),
            "algorand".to_string(),
            "near".to_string(),
            "amp-token".to_string(),
            "internet-computer".to_string(),
            "filecoin".to_string(),
            "compound-coin".to_string(),
            "fetch-ai".to_string(),
            "cardano".to_string(),
            "gitcoin".to_string(),
            "nucypher".to_string(),
            "solana".to_string(),
            "tether".to_string(),
            "gallant".to_string(),
            "dai".to_string(),
            "polkadot".to_string(),
            "usd-coin".to_string(),
            "storj".to_string(),
            "tezos".to_string(),
            "skale".to_string(),
            "ankr".to_string(),
            "ampleforth-governance-token".to_string(),
            "matic-network".to_string(),
            "the-graph".to_string(),
            "chain-2".to_string(),
            "stellar".to_string(),
            "cosmos".to_string(),
            "polkastarter".to_string(),
        ];

        let ids: Vec<String> = tickers
            .iter()
            .map(|ticker| ticker_to_id(ticker).unwrap())
            .collect();

        assert_eq!(ids, expected)
    }

    #[test]
    fn converts_staking_tickers_to_ids() {
        let tickers = ["eth2.s", "algo.s", "ada.s", "sol.s", "dot.s", "atom.s"];

        let expected = vec![
            "ethereum".to_string(),
            "algorand".to_string(),
            "cardano".to_string(),
            "solana".to_string(),
            "polkadot".to_string(),
            "cosmos".to_string(),
        ];

        let ids: Vec<String> = tickers
            .iter()
            .map(|ticker| ticker_to_id(ticker).unwrap())
            .collect();

        assert_eq!(ids, expected)
    }
}
