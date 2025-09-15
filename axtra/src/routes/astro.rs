use axum::{
    RequestPartsExt, Router,
    body::Body,
    extract::{OriginalUri, Request},
    response::IntoResponse,
    routing::get,
};
use http::{StatusCode, header};
use tower::ServiceExt;
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};

pub fn serve_spa<S>(path: impl AsRef<str>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let path = path.as_ref();
    let index_file_path = format!("./dist/{path}/index.html");

    let serve_index = {
        let index_file_path = index_file_path.clone();
        move |req: Request<Body>| {
            let index_file_path = index_file_path.clone();
            async move {
                let serve_file = ServeFile::new(index_file_path.clone());
                let mut res = serve_file.oneshot(req).await.into_response();

                // Force no-cache for index.html
                res.headers_mut().insert(
                    header::CACHE_CONTROL,
                    "no-cache, no-store, must-revalidate".parse().unwrap(),
                );
                res.headers_mut()
                    .insert(header::PRAGMA, "no-cache".parse().unwrap());
                res.headers_mut()
                    .insert(header::EXPIRES, "0".parse().unwrap());

                res
            }
        }
    };

    Router::new()
        .route(&format!("/{path}"), get(serve_index.clone()))
        .route(&format!("/{path}/{{*route}}"), get(serve_index))
}

pub fn serve_static_files<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let public_path = "./dist";
    let fallback_service = ServeDir::new(public_path)
        .append_index_html_on_directories(true)
        .not_found_service(ServeFile::new(format!("{}/{}", public_path, "404.html")));
    let compression_layer: CompressionLayer = CompressionLayer::new().gzip(true);

    // Base router
    Router::new()
        .fallback(get(|req: Request| async move {
            let (mut parts, body) = req.into_parts();
            let uri: OriginalUri = parts.extract().await?;

            let req = Request::from_parts(parts, body);
            match fallback_service.oneshot(req).await {
                Ok(mut res) => match res.status() {
                    StatusCode::OK => {
                        if uri.path().contains("/_static/") {
                            res.headers_mut().insert(
                                header::CACHE_CONTROL,
                                // One year cache
                                "public, max-age=31536000".parse().unwrap(),
                            );
                        }
                        if uri.path().contains("/_astro/") {
                            res.headers_mut().insert(
                                header::CACHE_CONTROL,
                                // One month cache
                                "public, max-age=2628000".parse().unwrap(),
                            );
                        }
                        Ok(res)
                    }
                    _ => Ok(res),
                },
                Err(e) => {
                    tracing::error!("Static file serve error: {e}");
                    Err(e)
                }
            }
        }))
        .layer(compression_layer)
}
