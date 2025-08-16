use serde_json::Value;
use std::collections::HashMap;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use kiteconnect_rs::constants::Endpoints;

pub struct ApiEndpointMappings;

impl ApiEndpointMappings {
    pub fn get_endpoints() -> HashMap<(&'static str, &'static str), &'static str> {
        let mut endpoints = HashMap::new();

        // Format: (HTTP_METHOD, PATH) -> MOCK_FILE
        // User endpoints
        endpoints.insert(("GET", Endpoints::USER_PROFILE), "profile.json");
        endpoints.insert(("GET", Endpoints::USER_FULL_PROFILE), "full_profile.json");
        endpoints.insert(("GET", Endpoints::USER_MARGINS), "margins.json");
        endpoints.insert(("GET", "/user/margins/equity"), "margins_equity.json"); // Specific segment for testing

        // Session endpoints
        endpoints.insert(
            ("POST", Endpoints::SESSION_GENERATE),
            "generate_session.json",
        );
        endpoints.insert(
            ("DELETE", Endpoints::INVALIDATE_TOKEN),
            "session_logout.json",
        );
        endpoints.insert(("POST", Endpoints::RENEW_ACCESS), "generate_session.json"); // Using same file for refresh token as it has the required fields

        // Portfolio endpoints
        endpoints.insert(("GET", Endpoints::GET_POSITIONS), "positions.json");
        endpoints.insert(("GET", Endpoints::GET_HOLDINGS), "holdings.json");
        endpoints.insert(
            ("POST", Endpoints::INIT_HOLDINGS_AUTH),
            "holdings_auth.json",
        );
        endpoints.insert(
            ("GET", Endpoints::AUCTION_INSTRUMENTS),
            "auctions_list.json",
        );
        endpoints.insert(
            ("PUT", Endpoints::CONVERT_POSITION),
            "convert_position.json",
        );

        // Order endpoints
        endpoints.insert(("GET", Endpoints::GET_ORDERS), "orders.json");
        endpoints.insert(("GET", Endpoints::GET_TRADES), "trades.json");
        endpoints.insert(("GET", "/orders/151220000000000"), "order_info.json"); // Mock order ID
        endpoints.insert(
            ("GET", "/orders/151220000000000/trades"),
            "order_trades.json",
        ); // Mock order ID
        endpoints.insert(("POST", "/orders/regular"), "order_response.json"); // Mock variety
        endpoints.insert(("POST", "/orders/iceberg"), "order_response.json"); // Mock variety
        endpoints.insert(("POST", "/orders/co"), "order_response.json"); // Mock variety
        endpoints.insert(("POST", "/orders/auction"), "order_response.json"); // Mock variety
        endpoints.insert(
            ("PUT", "/orders/regular/151220000000000"),
            "order_modify.json",
        ); // Mock variety and order ID
        endpoints.insert(
            ("DELETE", "/orders/regular/151220000000000"),
            "order_response.json",
        ); // Mock variety and order ID

        // Mutual Fund endpoints
        endpoints.insert(("GET", Endpoints::GET_MF_ORDERS), "mf_orders.json");
        endpoints.insert(("GET", "/mf/orders/test"), "mf_orders_info.json"); // Mock order ID
        endpoints.insert(("POST", Endpoints::PLACE_MF_ORDER), "order_response.json"); // Use existing order response format
        endpoints.insert(("DELETE", "/mf/orders/test"), "order_response.json"); // Mock order ID - use existing format
        endpoints.insert(("GET", Endpoints::GET_MF_SIPS), "mf_sips.json");
        endpoints.insert(("GET", "/mf/sips/test"), "mf_sip_info.json"); // Mock SIP ID
        endpoints.insert(("POST", Endpoints::PLACE_MF_SIP), "mf_sip_place.json");
        endpoints.insert(("PUT", "/mf/sips/test"), "mf_sip_info.json"); // Use mf_sip_info.json as per Go mapping
        endpoints.insert(("DELETE", "/mf/sips/test"), "mf_sip_cancel.json"); // Mock SIP ID
        endpoints.insert(("GET", Endpoints::GET_MF_HOLDINGS), "mf_holdings.json");
        endpoints.insert(("GET", "/mf/holdings/test"), "mf_holdings.json"); // Mock ISIN - for now, we'll handle the type mismatch in tests
        endpoints.insert(
            ("GET", Endpoints::GET_MF_ALLOTTED_ISINS),
            "mf_holdings.json",
        ); // For now, we'll handle the type mismatch in tests

        // Margin endpoints
        endpoints.insert(("POST", Endpoints::ORDER_MARGINS), "order_margins.json");
        endpoints.insert(("POST", Endpoints::BASKET_MARGINS), "basket_margins.json");
        endpoints.insert(
            ("POST", Endpoints::ORDER_CHARGES),
            "virtual_contract_note.json",
        );

        // Market data endpoints
        endpoints.insert(("GET", Endpoints::GET_QUOTE), "quote.json");
        endpoints.insert(("GET", Endpoints::GET_LTP), "ltp.json");
        endpoints.insert(("GET", Endpoints::GET_OHLC), "ohlc.json");
        endpoints.insert(
            ("GET", "/instruments/historical/123/myinterval"),
            "historical_minute.json",
        ); // Mock instrument token and interval
        endpoints.insert(
            ("GET", "/instruments/historical/456/myinterval"),
            "historical_oi.json",
        ); // Mock instrument token and interval with OI
        endpoints.insert(
            ("GET", "/instruments/NSE/INFY/trigger_range"),
            "trigger_range.json",
        ); // Mock exchange and tradingsymbol

        // Alerts API endpoints
        endpoints.insert(("POST", "/alerts"), "alerts_create.json");
        endpoints.insert(("GET", "/alerts"), "alerts_get.json");
        endpoints.insert(
            ("GET", "/alerts/550e8400-e29b-41d4-a716-446655440000"),
            "alerts_get_one.json",
        );
        endpoints.insert(
            ("PUT", "/alerts/550e8400-e29b-41d4-a716-446655440000"),
            "alerts_modify.json",
        );
        endpoints.insert(("DELETE", "/alerts"), "alerts_delete.json");
        endpoints.insert(
            (
                "GET",
                "/alerts/550e8400-e29b-41d4-a716-446655440000/history",
            ),
            "alerts_history.json",
        );

        endpoints
    }
}

pub struct KiteMockServer {
    pub server: MockServer,
    pub base_url: String,
}

impl KiteMockServer {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let base_url = server.uri();

        Self { server, base_url }
    }

    pub async fn setup_all_mocks(&self) {
        let endpoints = ApiEndpointMappings::get_endpoints();

        for ((http_method, endpoint_path), mock_file) in endpoints {
            let mock_data = Self::load_mock_data(mock_file);

            Mock::given(method(http_method))
                .and(path(endpoint_path))
                .respond_with(ResponseTemplate::new(200).set_body_json(mock_data))
                .mount(&self.server)
                .await;
        }

        // Setup CSV endpoints separately
        self.setup_csv_mocks().await;
    }

    pub async fn setup_csv_mocks(&self) {
        // Instruments endpoints return CSV, not JSON
        Mock::given(method("GET"))
            .and(path(Endpoints::GET_INSTRUMENTS))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(Self::load_csv_data("instruments_all.csv"))
                    .insert_header("content-type", "text/csv"),
            )
            .mount(&self.server)
            .await;

        Mock::given(method("GET"))
            .and(path("/instruments/nse"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(Self::load_csv_data("instruments_nse.csv"))
                    .insert_header("content-type", "text/csv"),
            )
            .mount(&self.server)
            .await;

        Mock::given(method("GET"))
            .and(path(Endpoints::GET_MF_INSTRUMENTS))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(Self::load_csv_data("mf_instruments.csv"))
                    .insert_header("content-type", "text/csv"),
            )
            .mount(&self.server)
            .await;
    }

    pub fn load_csv_data(filename: &str) -> String {
        let mock_path = format!("tests/mocks/{}", filename);
        std::fs::read_to_string(&mock_path)
            .unwrap_or_else(|_| panic!("Failed to read mock CSV file: {}", mock_path))
    }

    pub fn load_mock_data(filename: &str) -> Value {
        let mock_path = format!("tests/mocks/{}", filename);
        let mock_data = std::fs::read_to_string(&mock_path)
            .unwrap_or_else(|_| panic!("Failed to read mock file: {}", mock_path));

        serde_json::from_str(&mock_data)
            .unwrap_or_else(|_| panic!("Failed to parse JSON from: {}", mock_path))
    }
}
