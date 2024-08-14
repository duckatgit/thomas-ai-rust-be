use std::borrow::Cow;

use super::OpenApiFromData;
use super::gen::OpenApiGenerator;
use axum::{Json, http::Error, body::Bytes};
use okapi::{
    openapi3::{MediaType, RequestBody, SecurityScheme, Parameter, SecurityRequirement, Responses},
    Map,
};
use schemars::JsonSchema;
use serde::Deserialize;
use anyhow::Result;

macro_rules! fn_request_body {
    ($gen:ident, $ty:path, $mime_type:expr) => {{
        let schema = $gen.json_schema::<$ty>();
        Ok(RequestBody {
            content: {
                let mut map = Map::new();
                map.insert(
                    $mime_type.to_owned(),
                    MediaType {
                        schema: Some(schema),
                        ..MediaType::default()
                    },
                );
                map
            },
            required: true,
            ..okapi::openapi3::RequestBody::default()
        })
    }};
}

impl<'r, T: JsonSchema + Deserialize<'r>> OpenApiFromData<'r> for Json<T> {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        fn_request_body!(gen, T, "application/json")
    }
}

impl<'r> OpenApiFromData<'r> for String {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        fn_request_body!(gen, String, "application/octet-stream")
    }
}

impl<'r> OpenApiFromData<'r> for &'r str {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        fn_request_body!(gen, str, "application/octet-stream")
    }
}

impl<'r> OpenApiFromData<'r> for Cow<'r, str> {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        fn_request_body!(gen, str, "application/octet-stream")
    }
}

impl<'r> OpenApiFromData<'r> for Vec<u8> {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        fn_request_body!(gen, Vec<u8>, "application/octet-stream")
    }
}

impl<'r> OpenApiFromData<'r> for Bytes {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        fn_request_body!(gen, Vec<u8>, "application/octet-stream")
    }
}

impl<'r> OpenApiFromData<'r> for &'r [u8] {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        Vec::<u8>::request_body(gen)
    }
}

impl<'r, T: OpenApiFromData<'r> + 'r> OpenApiFromData<'r> for Result<T, Error> {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        T::request_body(gen)
    }
}

impl<'r, T: OpenApiFromData<'r>> OpenApiFromData<'r> for Option<T> {
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody> {
        Ok(RequestBody {
            required: false,
            ..T::request_body(gen)?
        })
    }
}


#[allow(clippy::large_enum_variant)]
pub enum RequestHeaderInput {
    /// This request header requires no input anywhere
    None,
    /// Useful for when you want to set a header per route.
    Parameter(Parameter),
    /// The request guard implements a security scheme.
    ///
    /// Parameters:
    /// - The name of the [`SecurityScheme`].
    /// - [`SecurityScheme`] is global definition of the authentication (per OpenApi spec).
    /// - [`SecurityRequirement`] is the requirements for the route.
    Security(String, SecurityScheme, SecurityRequirement),
}

/// Trait that needs to be implemented for all types that implement
/// [`FromRequest`](rocket::request::FromRequest).
/// This trait specifies what headers or other parameters are required for this
/// [Request Guards](https://rocket.rs/v0.5-rc/guide/requests/#request-guards)
/// to be validated successfully.
///
/// If it does not quire any headers or parameters you can use the derive macro:
/// ```rust,ignore
/// use rocket_okapi::request::OpenApiFromRequest;
///
/// #[derive(OpenApiFromRequest)]
/// pub struct MyStructName;
/// ```
pub trait OpenApiFromRequest<B>: axum::extract::FromRequest<B> {
    /// Specifies what headers or other parameters are required for this Request Guards to validate
    /// successfully.
    fn from_request_input(
        gen: &mut OpenApiGenerator,
        name: String,
        required: bool,
    ) -> Result<RequestHeaderInput>;

    /// Optionally add responses to the Request Guard.
    /// This can be used for when the request guard could return a "401 Unauthorized".
    /// Or any other responses, other then one from the default response.
    fn get_responses(_gen: &mut OpenApiGenerator) -> Result<Responses> {
        Ok(Responses::default())
    }
}
