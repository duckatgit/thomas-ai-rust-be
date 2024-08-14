use std::fmt::Display;

use anyhow::Result;
use axum::http::Method;
use gen::OpenApiGenerator;
use okapi::openapi3::{Object, Parameter, ParameterValue, RequestBody, SchemaObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OpenApiMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Head,
    Trace,
    // Connect not available in OpenAPI3. Maybe should set in extensions?
    Connect,
}

impl Display for OpenApiMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenApiMethod::Get => write!(f, "{}", "GET"),
            OpenApiMethod::Post => write!(f, "{}", "POST"),
            OpenApiMethod::Put => write!(f, "{}", "PUT"),
            OpenApiMethod::Patch => write!(f, "{}", "PATCH"),
            OpenApiMethod::Delete => write!(f, "{}", "DELETE"),
            OpenApiMethod::Head => write!(f, "{}", "HEAD"),
            OpenApiMethod::Connect => write!(f, "{}", "CONNECT"),
            OpenApiMethod::Options => write!(f, "{}", "OPTIONS"),
            OpenApiMethod::Trace => write!(f, "{}", "TRACE"),
        }
    }
}

pub mod error;
pub mod gen;
pub mod request;
pub mod response;
pub mod settings;
pub mod utils;

#[macro_use]
pub use openapi_proc_macro;

impl Into<OpenApiMethod> for Method {
    fn into(self) -> OpenApiMethod {
        match self {
            Method::GET => OpenApiMethod::Get,
            Method::POST => OpenApiMethod::Post,
            Method::PUT => OpenApiMethod::Put,
            Method::PATCH => OpenApiMethod::Patch,
            Method::DELETE => OpenApiMethod::Delete,
            Method::HEAD => OpenApiMethod::Head,
            Method::CONNECT => OpenApiMethod::Connect,
            Method::OPTIONS => OpenApiMethod::Options,
            Method::TRACE => OpenApiMethod::Trace,
            _ => OpenApiMethod::Get,
        }
    }
}

impl Into<OpenApiMethod> for &str {
    fn into(self) -> OpenApiMethod {
        match self {
            "GET" => OpenApiMethod::Get,
            "POST" => OpenApiMethod::Post,
            "PUT" => OpenApiMethod::Put,
            "PATCH" => OpenApiMethod::Patch,
            "DELETE" => OpenApiMethod::Delete,
            "HEAD" => OpenApiMethod::Head,
            "CONNECT" => OpenApiMethod::Connect,
            "OPTIONS" => OpenApiMethod::Options,
            "TRACE" => OpenApiMethod::Trace,
            _ => OpenApiMethod::Get,
        }
    }
}

impl Into<OpenApiMethod> for String {
    fn into(self) -> OpenApiMethod {
        match self.as_str() {
            "GET" => OpenApiMethod::Get,
            "POST" => OpenApiMethod::Post,
            "PUT" => OpenApiMethod::Put,
            "PATCH" => OpenApiMethod::Patch,
            "DELETE" => OpenApiMethod::Delete,
            "HEAD" => OpenApiMethod::Head,
            "CONNECT" => OpenApiMethod::Connect,
            "OPTIONS" => OpenApiMethod::Options,
            "TRACE" => OpenApiMethod::Trace,
            _ => OpenApiMethod::Get,
        }
    }
}

pub struct OperationInfo {
    /// The path of the endpoint
    pub path: String,
    /// The HTTP Method of this endpoint.
    pub method: OpenApiMethod,
    /// Contains information to be showed in the documentation about this endpoint.
    pub operation: okapi::openapi3::Operation,
}

pub fn parameter_from_schema(schema: SchemaObject, name: String, mut required: bool) -> Parameter {
    // Check if parameter is optional (only is not already optional)
    if required {
        for (key, value) in &schema.extensions {
            if key == "nullable" {
                if let Some(nullable) = value.as_bool() {
                    required = !nullable;
                }
            }
        }
    }
    let description = schema.metadata.as_ref().and_then(|m| m.description.clone());
    Parameter {
        name,
        location: "query".to_owned(),
        description,
        required,
        deprecated: false,
        allow_empty_value: false,
        value: ParameterValue::Schema {
            style: None,
            explode: None,
            allow_reserved: false,
            schema,
            example: None,
            examples: None,
        },
        extensions: Object::default(),
    }
}

pub trait OpenApiFromData<'r> {
    /// Return a [`RequestBody`] containing the information required to document the
    /// [`FromData`](rocket::data::FromData) object.
    fn request_body(gen: &mut OpenApiGenerator) -> Result<RequestBody>;
}
