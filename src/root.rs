use url::Url;

/// Name of query parameter that is used to specify the endpoint route when
/// pretty permalinks is not enabled on a WordPress instance.
const REST_ROUTE_QUERY_PARAM: &str = "rest_route";

/// API root route.
#[derive(Debug)]
pub enum RootRoute {
    /// The default root for WordPress instances without "pretty permalinks"
    /// enabled.
    ///
    /// API endpoint routes are added to the URL as the value for the
    /// "rest_route" query parameter.
    Default(Url),

    /// Pretty permalinks root
    PrettyPermalinks(Url),
}

impl RootRoute {
    pub fn as_str(&self) -> &str {
        let url = match self {
            RootRoute::Default(url) => url,
            RootRoute::PrettyPermalinks(url) => url,
        };
        url.as_str()
    }

    /// Join an endpoint route onto the root route
    pub fn join(&self, route: &str) -> Url {
        match self {
            RootRoute::Default(url) => {
                let prev_pairs = url.query_pairs();
                let mut url = url.clone();
                {
                    let mut query_pairs = url.query_pairs_mut();
                    query_pairs.clear();
                    for (key, value) in prev_pairs {
                        if key == REST_ROUTE_QUERY_PARAM {
                            query_pairs.append_pair(REST_ROUTE_QUERY_PARAM, route);
                        } else if value == "" {
                            query_pairs.append_key_only(&key);
                        } else {
                            query_pairs.append_pair(&key, &value);
                        }
                    }
                }
                url
            }
            RootRoute::PrettyPermalinks(url) => {
                let mut url = url.clone();
                url.path_segments_mut()
                    .unwrap()
                    .pop_if_empty()
                    .extend(route.trim_start_matches('/').split('/'));
                url
            }
        }
    }
}

impl From<Url> for RootRoute {
    fn from(url: Url) -> Self {
        if url
            .query_pairs()
            .any(|(param, _)| param == REST_ROUTE_QUERY_PARAM)
        {
            RootRoute::Default(url)
        } else {
            RootRoute::PrettyPermalinks(url)
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    use super::*;

    #[test_case("http://example.com/wp-json/"      => matches RootRoute::PrettyPermalinks(_) ; "pretty permalinks")]
    #[test_case("http://example.com/?rest_route=/" => matches RootRoute::Default(_)          ; "default")]
    fn from_url(url: &str) -> RootRoute {
        Url::parse(url).unwrap().into()
    }

    #[test]
    fn join_pretty_permalinks() {
        let root: RootRoute = Url::parse("http://example.com/wp-json/").unwrap().into();
        let url = root.join("/wp/v2/posts/1");
        assert_eq!(url.as_str(), "http://example.com/wp-json/wp/v2/posts/1")
    }

    #[test]
    fn join_default() {
        let root: RootRoute = Url::parse("http://example.com/?rest_route=/")
            .unwrap()
            .into();
        let url = root.join("/wp/v2/posts/1");
        assert_eq!(
            url.as_str(),
            "http://example.com/?rest_route=%2Fwp%2Fv2%2Fposts%2F1"
        )
    }
}
