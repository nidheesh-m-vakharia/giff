use giff_github::{CreatePrParams, ForgeBackend, GitHubForge};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn create_pr_returns_pr_number() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/repos/owner/repo/pulls"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "number": 99,
            "html_url": "https://github.com/owner/repo/pull/99",
            "state": "open"
        })))
        .mount(&server)
        .await;

    let forge = GitHubForge::new("fake-token".into(), "owner/repo".into(), server.uri());
    let pr = forge
        .create_pr(CreatePrParams {
            title: "feat: add auth".into(),
            body: "description".into(),
            head: "feat/auth".into(),
            base: "main".into(),
            draft: false,
        })
        .unwrap();

    assert_eq!(pr.number, 99);
    assert_eq!(pr.html_url, "https://github.com/owner/repo/pull/99");
}

#[tokio::test]
async fn update_pr_updates_body() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/repos/owner/repo/pulls/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "number": 99,
            "html_url": "https://github.com/owner/repo/pull/99",
            "state": "open"
        })))
        .mount(&server)
        .await;

    let forge = GitHubForge::new("fake-token".into(), "owner/repo".into(), server.uri());
    let pr = forge
        .update_pr(
            99,
            giff_github::UpdatePrParams {
                body: Some("new body".into()),
                base: None,
            },
        )
        .unwrap();

    assert_eq!(pr.number, 99);
}
