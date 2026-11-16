use mm_core::BridgeStatus;

pub fn index_html(status: &BridgeStatus) -> String {
    let status_json = serde_json::to_string_pretty(status)
        .unwrap_or_else(|_| "{\"error\":\"failed to encode status\"}".to_string());

    let escaped_json = escape_html(&status_json);
    let version = escape_html(&status.bridge.version);
    let server_count = status.servers.len();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>emacs-mattermost bridge</title>
  <style>
    body {{
      font-family: system-ui, sans-serif;
      margin: 2rem;
      background: #111827;
      color: #e5e7eb;
    }}
    a {{
      color: #93c5fd;
    }}
    pre {{
      background: #030712;
      border: 1px solid #374151;
      padding: 1rem;
      overflow: auto;
    }}
    .ok {{
      color: #86efac;
      font-weight: bold;
    }}
  </style>
</head>
<body>
  <h1>emacs-mattermost bridge</h1>
  <p>Status: <span class="ok">running</span></p>
  <p>Version: {version}</p>
  <p>Configured servers: {server_count}</p>
  <p>
    <a href="/health">/health</a> |
    <a href="/status">/status</a>
  </p>
  <h2>Status JSON</h2>
  <pre>{escaped_json}</pre>
</body>
</html>
"#
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
