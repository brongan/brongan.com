use crate::mandelbrot::mandelbrot::MandelbrotRequest;
use crate::mandelbrot::mandelbrot::MandelbrotResponse;
use axum::extract::{Query, State};
use axum::{
    http::header::HeaderMap,
    response::{IntoResponse, Json, Response},
};
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use shared::mandelbrot;
use shared::mandelbrot::generate_mandelbrot_multithreaded;
use std::io::Cursor;

use crate::ServerState;

#[axum::debug_handler]
pub async fn mandelbrot_get(
    _headers: HeaderMap,
    State(_state): State<ServerState>,
    args: Query<MandelbrotRequest>,
) -> Response {
    let tracer = global::tracer("");
    let _ = tracer.start("mandelbrot_get");
    let mandelbrot =
        generate_mandelbrot_multithreaded(args.bounds, args.upper_left, args.lower_right);

    let mut image: Vec<u8> = Vec::new();
    mandelbrot
        .write_to(&mut Cursor::new(&mut image), image::ImageOutputFormat::Png)
        .unwrap();

    Json(MandelbrotResponse { image }).into_response()
}
