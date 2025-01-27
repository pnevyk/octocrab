use std::sync::Arc;

use http::{header::AUTHORIZATION, request::Request, HeaderValue};
use tower::{Layer, Service};

#[derive(Clone)]
/// Layer that adds the authentication header to github-bound requests
pub struct AuthHeaderLayer {
    pub(crate) auth_header: Arc<Option<HeaderValue>>,
}

impl AuthHeaderLayer {
    pub fn new(auth_header: Option<HeaderValue>) -> Self {
        AuthHeaderLayer {
            auth_header: Arc::new(auth_header),
        }
    }
}

impl<S> Layer<S> for AuthHeaderLayer {
    type Service = AuthHeader<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthHeader {
            inner,
            auth_header: self.auth_header.clone(),
        }
    }
}

#[derive(Clone)]
/// Service that adds a static set of extra headers to each request
pub struct AuthHeader<S> {
    inner: S,
    pub(crate) auth_header: Arc<Option<HeaderValue>>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for AuthHeader<S>
where
    S: Service<Request<ReqBody>>,
{
    type Error = S::Error;
    type Future = S::Future;
    type Response = S::Response;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Only set the auth_header if the authority (host) is empty (destined for
        // GitHub). Otherwise, leave it off as we could have been redirected
        // away from GitHub (via follow_location_to_data()), and we don't
        // want to give our credentials to third-party services.
        if req.uri().authority().is_none() {
            if let Some(auth_header) = &*self.auth_header {
                req.headers_mut().append(AUTHORIZATION, auth_header.clone());
            }
        }
        self.inner.call(req)
    }
}
