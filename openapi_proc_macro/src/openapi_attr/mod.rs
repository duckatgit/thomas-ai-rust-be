extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use darling::FromMeta;
use proc_macro2::Span;
// use proc_macro2::TokenStream;
use proc_macro::TokenStream;
use syn::AttributeArgs;
use std::collections::HashMap;
use std::ops::Fn;
use syn::parse;
use syn::parse_macro_input;
use syn::Data;
use syn::DeriveInput;
use syn::FnArg;
use syn::GenericArgument;
use syn::Ident;
use syn::ItemFn;
use syn::PathArguments;
use syn::Type;
#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct OpenApiAttribute {
    pub method: String,

    pub summary: String,

    pub description: String,

    #[darling(multiple, rename = "tag")]
    pub tags: Vec<String>,

    #[darling(multiple, rename = "headers")]
    pub headers: Vec<String>,
}

pub fn parse_query(_: TokenStream, input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse(input.into()).expect("failed to parse input");

    let name = &ast.ident;
    let data = &ast.data;

    let mut fields = vec![];

    match data {
        Data::Struct(s) => {
            for field in &s.fields {
                if let Some(ident) = &field.ident {
                    fields.push(ident);
                }
            }
        }
        _ => {}
    }

    quote! {
        impl #name {
            pub fn query( gen: &mut OpenApiGenerator) -> Vec<RefOr<Parameter>> {
                use ::openapi_rs::parameter_from_schema;
                use ::okapi::openapi3::{Object, Parameter, ParameterValue};
                use ::schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};
                use ::schemars::JsonSchema;

                let schema = gen.json_schema_no_ref::<#name>();
    // Get a list of properties from the structure.
    let mut properties: schemars::Map<String, Schema> = schemars::Map::new();
    // Create all the `Parameter` for every property
    let mut parameter_list: Vec<RefOr<Parameter>> = Vec::new();
    match &schema.instance_type {
        Some(SingleOrVec::Single(instance_type)) => {
            if **instance_type == InstanceType::Object {
                if let Some(object) = schema.object {
                    properties = object.properties;
                }
                for (key, property) in properties {
                    let prop_schema = match property {
                        Schema::Object(x) => x,
                        _ => SchemaObject::default(),
                    };
                    parameter_list.push(RefOr::Object(parameter_from_schema(prop_schema, key, true)));
                }
            } else {
                #(parameter_list.push(RefOr::Object(parameter_from_schema(schema.clone(), stringify!(#fields).to_string(), true))));*
                
            }
        }
        None => {
            // Used when `SchemaObject.reference` is set.
            // https://github.com/GREsau/schemars/issues/105
            #(parameter_list.push(RefOr::Object(parameter_from_schema(schema.clone(), stringify!(#fields).to_string(), true))));*
        }
        _ => {
            // TODO: Do nothing for now, might need implementation later.
            // log::warn!(
            //     "Please let `okapi` devs know how you triggered this type: `{:?}`.",
            //     schema.instance_type
            // );
            #(parameter_list.push(RefOr::Object(parameter_from_schema(schema.clone(), stringify!(#fields).to_string(), true))));*
        }
    }
    parameter_list
            }
        }

    }
    .into()
}

fn extract_generic_ident(arguments: &PathArguments) -> Vec<Ident> {
    let mut idents = vec![];

    if let PathArguments::AngleBracketed(angle_ga) = &arguments {
        if let Some(ga) = angle_ga.args.first() {
            
            if let GenericArgument::Type(Type::Path(ty)) = ga {
                
               let generic_segments = &ty.path.segments;

                for generic_segment in generic_segments {
                    idents.push(generic_segment.ident.clone());
                    if !generic_segment.arguments.is_empty() {
                        for ident in extract_generic_ident(&generic_segment.arguments) {
                            idents.push(ident);
                        }
                    }
                }

            }

        }
    }

   idents
}

fn filter_type<P: Fn(&Ident) -> bool>(ty: Box<Type>, cb: P) -> Vec<Ident> {
    let mut idents = vec![];

    match &*ty {
        Type::Path(p) => {
            let segments = &p.path.segments;

            for segment in segments {
                let ident = &segment.ident;

                let generic_idents = extract_generic_ident(&segment.arguments);

                if cb(ident) {
                    idents.push(ident.clone());
                    for ident in generic_idents {
                            idents.push(ident);
                    }
                }
            }
        }
        _ => {}
    }

    idents
}

pub fn parse_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    // let args1: proc_macro::TokenStream  = args.into();
    let attr_args = parse_macro_input!(args as AttributeArgs);
    // let attr_args = parse_macro_input!(args as AttributeArgs);
    // let input = parse_macro_input!(input as ItemFn);
    let input_fn: ItemFn = parse(input.into()).expect("failed to parse input");

    let okapi_attr = match OpenApiAttribute::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let mut tokens_map = HashMap::new();

    let fn_args = &input_fn.sig.inputs;

    if fn_args.len() > 0 {
        for fn_arg in fn_args {
            if let FnArg::Typed(t) = fn_arg {
                let query_idents = filter_type(t.ty.clone(), |i| i == "Query");
                let request_idents = filter_type(t.ty.clone(), |i| i.to_string().contains("Result") || i == "Json");
                let header_idents = filter_type(t.ty.clone(), |i| i == "TypedHeader");
                let auth_idents = filter_type(t.ty.clone(),|i| i.to_string().contains("Auth"));
    
                if let Some(query_name) = query_idents.last() {
                    let q = quote! {
                        let mut parameters = #query_name::query(gen);
                    };
                    tokens_map.insert("query",q);
                } else {
                    if !tokens_map.contains_key("query") {
                        let q = quote! {
                            let mut parameters: Vec<RefOr<Parameter>> = vec![];
                        };
                        tokens_map.insert("query",q);
                    }
                }
    
                let requests = request_idents.iter().any(|i| i.to_string().contains("Result"));
    
                let mut ident = request_idents.last();

                if requests {
                    ident = request_idents.get(2);
                }
    
                if let Some(request_name) = ident {
                    let b = quote! {
                        let requests = Some(RefOr::Object(Json::<#request_name>::request_body(gen).expect("failed to generate response schema")));
                    };
                    tokens_map.insert("request",b);
                } else {
                    if !tokens_map.contains_key("request") {
                    let b = quote! {
                        let requests: Option<RefOr<RequestBody>> = None;
                    };
                    tokens_map.insert("request",b);
                    }
                }
    
                if let Some(auth) = auth_idents.last() {
                    let a = quote! {
                        let scheme_type = <#auth as OpenApiFromRequest::<String>>::from_request_input(gen,String::new(),true);
                        let mut requirement = None;
                        let mut scheme = None;
                        let mut name = None;
    
                        if let Ok(RequestHeaderInput::Security(security_name,security_scheme,security_requirement)) = scheme_type {
                            name = Some(security_name);
                            requirement = Some(vec![security_requirement]);
                            scheme = Some(security_scheme); 
                        }
                    };
    
                    tokens_map.insert("auth",a);
                } else {
    
                    if !tokens_map.contains_key("auth") {
                        let a = quote! {
                            let requirement: Option<Vec<SecurityRequirement>> = None;
                            let scheme: Option<SecurityScheme> = None;
                            let name: Option<String> = None;
                        };    
    
                        tokens_map.insert("auth",a);
                    }
    
                }
    
    
                if let Some(header) = header_idents.last() {
                            let a = quote! {
                                let header_name = #header::name().as_str();
    
                                let schema = gen.json_schema::<String>();
    
                                let parameter = Parameter {
                                    name: header_name.into(),
                                    location: "header".to_owned(),
                                    description: None,
                                    required: true,
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
                                };
    
                                parameters.push(RefOr::Object(parameter));
    
                            };
        
                            tokens_map.insert("header",a);
    
                    } else {
                        if !tokens_map.contains_key("auth") {
                            let a = quote! {
                                let requirement: Option<Vec<SecurityRequirement>> = None;
                                let scheme: Option<SecurityScheme> = None;
                                let name: Option<String> = None;
                            };    
        
                            tokens_map.insert("auth",a);
                        }
                    }
            }
        }
    } else {
        let v = quote! {
            let requirement: Option<Vec<SecurityRequirement>> = None;
            let scheme: Option<SecurityScheme> = None;
            let name: Option<String> = None;
            let mut parameters: Vec<RefOr<Parameter>> = vec![];
            let requests: Option<RefOr<RequestBody>> = None;
        };

        tokens_map.insert("empty",v);
    }

    let return_type = input_fn.sig.output;

    let fn_ident = input_fn.sig.ident;

    let spec_fn_ident = Ident::new(&format!("{}_spec",&fn_ident),Span::call_site());

    match return_type {
        syn::ReturnType::Type(_, t) => {
            let body_ident = filter_type(t.clone(), |i| i.to_string().contains("Result") || i == "Json");

            if let Some(b_type) = body_ident.last() {
                let b = quote! {
                    let responses = Json::<#b_type>::responses(gen).expect("failed to generate response schema");
                };

                tokens_map.insert("response",b);
            }
        }
        _ => {}
    }

    let method = okapi_attr.method;
    let tags = okapi_attr.tags;
    let summary = okapi_attr.summary;
    let description = okapi_attr.description;

    let tokens = tokens_map.values();
    // println!("{:?}",return_type);
    quote! {
        pub fn #spec_fn_ident(path:&str,gen: &mut OpenApiGenerator) {
            use ::okapi::openapi3::{RequestBody,Parameter,Operation,SecurityScheme,SecurityRequirement,ParameterValue};
            use ::openapi_rs::OperationInfo;
            use ::openapi_rs::response::OpenApiResponderInner;
            use ::openapi_rs::request::{RequestHeaderInput,OpenApiFromRequest};
            use ::headers::*;

            #(#tokens)*

            if let (Some(security_name),Some(security_scheme)) = (name,scheme) {
                gen.add_security_scheme(security_name,security_scheme);
            }

            let operation = Operation {
                parameters,
                tags: vec![#(stringify!(#tags).to_string())*],
                summary: Some(#summary.to_string()),
                description: Some(#description.to_string()),
                responses,
                request_body: requests,
                operation_id: Some(stringify!(#fn_ident).to_string()),
                security: requirement,
                ..Default::default()
            };

            let mut operation_info = OperationInfo {
                path: path.to_string(),
                method: #method.into(),
                operation
            };

            gen.add_operation(operation_info);
        }
    }.into()
}
