use axum::Json;
use okapi::{openapi3::{Responses}};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{gen::OpenApiGenerator, utils::{produce_any_responses, add_schema_response}};

use anyhow::Result;

use super::utils::{ensure_status_code_exists};


pub trait OpenApiResponderInner {
    /// Create the responses type, which is a list of responses that can be
    /// rendered in `openapi.json` format.
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses>;
}

impl<T: Serialize + JsonSchema + Send> OpenApiResponderInner for Json<T> {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<T>();
        add_schema_response(&mut responses, 200, "application/json", schema)?;
        // 500 status is not added because an endpoint can handle this, so it might never return
        // this error type.
        Ok(responses)
    }
}

impl OpenApiResponderInner for &str {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        <String>::responses(gen)
    }
}

impl OpenApiResponderInner for String {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<String>();
        add_schema_response(&mut responses, 200, "text/plain", schema)?;
        Ok(responses)
    }
}

impl OpenApiResponderInner for &[u8] {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        <Vec<u8>>::responses(gen)
    }
}

impl OpenApiResponderInner for Vec<u8> {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<Vec<u8>>();
        add_schema_response(&mut responses, 200, "application/octet-stream", schema)?;
        Ok(responses)
    }
}

impl OpenApiResponderInner for std::fs::File {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        <Vec<u8>>::responses(gen)
    }
}


impl OpenApiResponderInner for () {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses> {
        let mut responses = Responses::default();
        ensure_status_code_exists(&mut responses, 200);
        Ok(responses)
    }
}

impl<T: OpenApiResponderInner> OpenApiResponderInner for Option<T> {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        let mut responses = T::responses(gen)?;
        ensure_status_code_exists(&mut responses, 404);
        Ok(responses)
    }
}

impl<'r, 'o: 'r, T> OpenApiResponderInner for std::borrow::Cow<'o, T>
where
    T: OpenApiResponderInner + Clone,
{
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        let mut responses = T::responses(gen)?;
        ensure_status_code_exists(&mut responses, 200);
        Ok(responses)
    }
}

impl<'r, 'o, T, E> OpenApiResponderInner for std::result::Result<T, E>
where
    T: OpenApiResponderInner,
    E: OpenApiResponderInner,
{
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses> {
        let ok_responses = T::responses(gen)?;
        let err_responses = E::responses(gen)?;
        produce_any_responses(ok_responses, err_responses)
    }
}