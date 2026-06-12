use url::Url;

const SENSITIVE_QUERY_KEYS: &[&str] = &[
    "access_token",
    "auth",
    "code",
    "id_token",
    "password",
    "token",
];

pub struct SanitizedUrl {
    pub sanitized: String,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

pub fn sanitize_url(raw: &str) -> SanitizedUrl {
    let Ok(mut url) = Url::parse(raw) else {
        return SanitizedUrl::from_raw(raw);
    };

    let _ = url.set_username("");
    let _ = url.set_password(None);
    redact_query(&mut url);

    SanitizedUrl {
        sanitized: url.to_string(),
        scheme: Some(url.scheme().to_string()),
        host: url.host_str().map(str::to_string),
        port: url.port_or_known_default(),
    }
}

fn redact_query(url: &mut Url) {
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| {
            let value = if SENSITIVE_QUERY_KEYS.contains(&key.as_ref()) {
                "REDACTED".into()
            } else {
                value
            };
            (key.into_owned(), value.into_owned())
        })
        .collect();

    url.query_pairs_mut().clear().extend_pairs(pairs);
}

impl SanitizedUrl {
    fn from_raw(raw: &str) -> Self {
        Self {
            sanitized: raw.to_string(),
            scheme: None,
            host: None,
            port: None,
        }
    }
}
