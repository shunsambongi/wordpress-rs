use std::error::Error;

use thiserror::Error;

/// Errors which may occur when using API endpoints.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// The client encountered an error.
    #[error("client error: {}", source)]
    Client {
        /// The client error.
        source: E,
    },

    /// URL failed to parse.
    #[error("failed to parse url: {}", source)]
    UrlParse {
        /// The source of the error.
        #[from]
        source: url::ParseError,
    },

    /// Failed to discover API root route.
    #[error("failed to discover root route")]
    RootRouteDiscovery,

    /// WordPress returned an error response.
    #[error("gitlab server error: [{}] {}", code, message)]
    WordPress {
        message: String,
        code: String,
        data: serde_json::Value,
    },

    /// WordPress returned an error without JSON information.
    #[error("wordpress internal server error {}", status)]
    WordPressInternal {
        /// The status code for the HTTP response.
        status: http::StatusCode,
        /// The error data from WordPress.
        data: Vec<u8>,
    },

    /// WordPress returned an HTTP error with JSON we did not recognize.
    #[error("wordpress server error: {:?}", json)]
    WordPressUnrecognized {
        /// The full JSON object from WordPress.
        json: serde_json::Value,
    },

    /// Failed to parse an expected data type from JSON.
    #[error("could not parse `{}` from JSON: {}", typename, source)]
    DataType {
        /// The source of the error.
        source: serde_json::Error,
        /// The name of the type that could not be deserialized.
        typename: &'static str,
    },
}

impl<E> ApiError<E>
where
    E: Error + Send + Sync + 'static,
{
    /// Create an API error from a client specific error.
    pub fn client(source: E) -> Self {
        ApiError::Client { source }
    }

    pub(crate) fn server_error(status: http::StatusCode, body: &bytes::Bytes) -> Self {
        Self::WordPressInternal {
            status,
            data: body.into_iter().copied().collect(),
        }
    }

    pub(crate) fn from_json(json: serde_json::Value) -> Self {
        let message = json.pointer("/message");
        let code = json.pointer("/code");
        let data = json.pointer("/data");

        let fields = match (message, code, data) {
            (Some(message), Some(code), Some(data)) => (message.as_str(), code.as_str(), data),
            _ => return ApiError::WordPressUnrecognized { json },
        };

        match fields {
            (Some(message), Some(code), data) => ApiError::WordPress {
                message: message.into(),
                code: code.into(),
                data: data.clone(),
            },
            _ => ApiError::WordPressUnrecognized { json },
        }
    }

    pub(crate) fn data_type<T>(source: serde_json::Error) -> Self {
        ApiError::DataType {
            source,
            typename: std::any::type_name::<T>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Error)]
    #[error("dummy")]
    struct Dummy;

    #[test]
    fn wordpress_error() {
        let obj = json!({
            "code": "rest_post_invalid_id",
            "message": "Invalid post ID.",
            "data": {
                "status": 404
            }
        });

        let err: ApiError<Dummy> = ApiError::from_json(obj);
        if let ApiError::WordPress {
            message,
            code,
            data: _data,
        } = err
        {
            assert_eq!(code, "rest_post_invalid_id");
            assert_eq!(message, "Invalid post ID.");
        } else {
            panic!("unexpected error: {}", err);
        }
    }

    #[test]
    fn wordpress_unrecognized() {
        let err_obj = json!({
            "bob": "loblaw"
        });

        let err: ApiError<Dummy> = ApiError::from_json(err_obj.clone());
        if let ApiError::WordPressUnrecognized { json: obj } = err {
            assert_eq!(obj, err_obj);
        } else {
            panic!("unexpected error: {}", err);
        }
    }
}
