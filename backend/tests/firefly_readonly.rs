use flow_finance_ai::firefly::FireflyClient;

#[test]
fn rejects_non_get_methods() {
    assert!(FireflyClient::reject_non_get("GET").is_ok());
    assert!(FireflyClient::reject_non_get("POST").is_err());
    assert!(FireflyClient::reject_non_get("PUT").is_err());
    assert!(FireflyClient::reject_non_get("DELETE").is_err());
}
