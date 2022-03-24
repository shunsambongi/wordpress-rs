use derive_builder::Builder;

use crate::endpoint::prelude::*;

#[derive(Builder)]
pub struct RetrievePost {
    id: u32,
}

impl RetrievePost {
    pub fn builder() -> RetrievePostBuilder {
        RetrievePostBuilder::default()
    }
}

impl Endpoint for RetrievePost {
    fn method(&self) -> Method {
        Method::GET
    }

    fn route(&self) -> Cow<'static, str> {
        format!("/wp/v2/posts/{}", self.id).into()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value as Json};

    use super::*;
    use crate::{
        test::{MockClient, MockResponse},
        Query,
    };

    #[tokio::test]
    async fn basic() {
        let endpoint = RetrievePost::builder().id(123).build().unwrap();
        let body = json!({
            "id": endpoint.id,
        });
        let response = MockResponse::builder()
            .method(endpoint.method())
            .route(endpoint.route())
            .json(body.clone())
            .build()
            .unwrap();
        let client = MockClient::with_response(response);

        let response: Json = endpoint.query(&client).await.unwrap();

        assert_eq!(response, body);
    }
}
