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

#[tokio::test]
async fn pr_status_returns_mergeable_and_draft() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "number": 99,
            "html_url": "https://github.com/owner/repo/pull/99",
            "state": "open",
            "mergeable": true,
            "draft": false
        })))
        .mount(&server)
        .await;

    let forge = GitHubForge::new("fake-token".into(), "owner/repo".into(), server.uri());
    let status = forge.pr_status(99).unwrap();

    assert_eq!(status.mergeable, Some(true));
    assert!(!status.draft);
}

#[tokio::test]
async fn pr_status_handles_null_mergeable() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/99"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "number": 99,
            "html_url": "https://github.com/owner/repo/pull/99",
            "state": "open",
            "mergeable": null,
            "draft": true
        })))
        .mount(&server)
        .await;

    let forge = GitHubForge::new("fake-token".into(), "owner/repo".into(), server.uri());
    let status = forge.pr_status(99).unwrap();

    assert_eq!(status.mergeable, None);
    assert!(status.draft);
}

#[tokio::test]
async fn get_pr_parses_merged_field() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/77"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "number": 77,
            "html_url": "https://github.com/owner/repo/pull/77",
            "state": "closed",
            "merged": true
        })))
        .mount(&server)
        .await;

    let forge = GitHubForge::new("fake-token".into(), "owner/repo".into(), server.uri());
    let pr = forge.get_pr(77).unwrap();

    assert_eq!(pr.number, 77);
    assert_eq!(pr.state, "closed");
    assert!(pr.merged);
}

#[tokio::test]
async fn list_open_pulls_parses_array_with_branch_refs() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "number": 1,
                "html_url": "https://github.com/owner/repo/pull/1",
                "state": "open",
                "title": "feat: a",
                "body": "Part 1/2 of stack `auth`.\n\n```giff\n{\"stack_id\":\"s1\",\"frame_id\":\"f1\",\"position\":1,\"total\":2}\n```",
                "head": { "ref": "feat/a" },
                "base": { "ref": "main" },
                "draft": false,
                "updated_at": "2026-04-29T12:00:00Z"
            },
            {
                "number": 2,
                "html_url": "https://github.com/owner/repo/pull/2",
                "state": "open",
                "title": "feat: b",
                "body": null,
                "head": { "ref": "feat/b" },
                "base": { "ref": "feat/a" },
                "draft": true,
                "updated_at": "2026-04-30T09:00:00Z"
            }
        ])))
        .mount(&server)
        .await;

    let forge = GitHubForge::new("fake-token".into(), "owner/repo".into(), server.uri());
    let pulls = forge.list_open_pulls().unwrap();

    assert_eq!(pulls.len(), 2);
    assert_eq!(pulls[0].number, 1);
    assert_eq!(pulls[0].title, "feat: a");
    assert_eq!(pulls[0].head.r#ref, "feat/a");
    assert_eq!(pulls[0].base.r#ref, "main");
    assert!(pulls[0].body.as_deref().unwrap().contains("```giff"));
    assert!(pulls[1].draft);
}

#[tokio::test]
async fn get_pr_defaults_merged_to_false_when_missing() {
    // The list endpoint doesn't return `merged` at all; the detail endpoint sometimes omits it
    // for never-mergeable PRs. Either way, deserialization must not fail.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo/pulls/55"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "number": 55,
            "html_url": "https://github.com/owner/repo/pull/55",
            "state": "open"
        })))
        .mount(&server)
        .await;

    let forge = GitHubForge::new("fake-token".into(), "owner/repo".into(), server.uri());
    let pr = forge.get_pr(55).unwrap();
    assert!(!pr.merged);
}
