//! Adapted from the Fastly Compute@Edge static content starter kit
//! See https://github.com/fastly/compute-starter-kit-rust-static-content

mod awsv4;

use std::str::Split;
use chrono::Utc;
use crate::awsv4::hash;
use fastly::handle::dictionary::DictionaryHandle;
use fastly::http::{header, Method, StatusCode};
use fastly::{Error, Request, Response};
use fastly::error::BufferKind::HeaderValue;
use lazy_static::lazy_static;
use regex::Regex;

/// Regex for extracting region from endpoint
lazy_static! {
    static ref REGION_REGEX: Regex = Regex::new(r"^s3\.([[:alnum:]\-]+)\.backblazeb2\.com$").unwrap();
}

// You must configure a backend named b2_backend
const B2_BACKEND: &str = "b2_origin";

const MAX_LEN_BOOLEAN: usize = 5;
const MAX_LEN_BUCKET_NAME: usize = 63;
const MAX_LEN_DOMAINNAME: usize = 253;
const MAX_LEN_APPLICATION_KEY_ID: usize = 25;
const MAX_LEN_APPLICATION_KEY: usize = 31;
const MAX_BUCKETS: usize = 100;

/// The entry point for the application.
///
/// This function is triggered when the service receives a client request.
/// It is used to route requests to a bucket in a specific region based on
/// the edge server ('pop') on which it is running.
///
/// If `main` returns an error, a 500 error response will be delivered to the client.
#[fastly::main]
fn main(mut req: Request) -> Result<Response, Error> {
    // Only allow GET and HEAD methods
    if ![Method::GET, Method::HEAD].contains(&req.get_method()) {
        return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED));
    }

    // Remove leading and trailing slashes from incoming path and split into segments
    let re = Regex::new(r"^/?(?P<path>.*?)/?$").unwrap();
    let path = re.replace(req.get_path(), "$path");
    let path_segments: Vec<&str> = path.split('/').collect();

    let config = match DictionaryHandle::open("config") {
        Ok(h) if h.is_valid() => h,
        _ => return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let allow_list_bucket = match config.get("allow_list_bucket", MAX_LEN_BOOLEAN) {
        Ok(Some(allow_list_bucket)) => allow_list_bucket.parse::<bool>().unwrap(),
        _ => return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let config_bucket_name = match config.get("bucket_name", MAX_LEN_BUCKET_NAME) {
        Ok(Some(bucket_name)) => bucket_name,
        _ => return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let bucket_list  = match config.get("allowed_buckets", MAX_LEN_BUCKET_NAME * MAX_BUCKETS) {
        Ok(Some(bucket_list)) => bucket_list,
        _ => return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    };
    let allowed_buckets: Vec<&str> = bucket_list.split(',').map(|bucket_name| bucket_name.trim()).collect();

    let endpoint = match config.get("endpoint", MAX_LEN_DOMAINNAME) {
        Ok(Some(endpoint)) => endpoint,
        _ => return Ok(Response::from_status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    if !allow_list_bucket {
        // Don't allow list bucket requests
        if (config_bucket_name == "$path" && path_segments.len() < 2)  // https://endpoint/bucket-name/
            || (config_bucket_name != "$path" && path.len() == 0) {
            return Ok(Response::from_status(StatusCode::NOT_FOUND));
        }
    }

    // Check access is allowed and normalize outgoing request path to /bucket-name/rest/of/path
    let (bucket_allowed, be_path) = match config_bucket_name.as_str() {
        // Bucket name is the first segment of the incoming path
        "$path" => {
            let bucket_name = path_segments[0];
            (
                allowed_buckets.contains(&bucket_name),
                format!("/{}", path)
            )
        },
        // Bucket name is incoming host prefix
        "$host" => {
            let bucket_name = req.get_url().host_str().unwrap().split('.').collect::<Vec<&str>>()[0];
            (
                allowed_buckets.contains(&bucket_name),
                format!("/{}/{}", bucket_name, path)
            )
        },
        // Bucket name is set in configuration
        _ => (
            true,
            format!("/{}/{}", config_bucket_name, path)
        ),
    };

    if !bucket_allowed {
        return Ok(Response::from_status(StatusCode::NOT_FOUND));
    }

    // Override incoming host header
    req.set_header(header::HOST, &endpoint);

    // Copy the modified client request to form the backend request.
    let mut be_req = req.clone_without_body();

    be_req.set_path(be_path.as_str());

    // Set the AWS V4 authentication headers
    sign_request(&mut be_req, endpoint);

    // Send the request to the backend
    let be_resp = be_req.send(B2_BACKEND)?;

    // return the response to the client.
    return Ok(be_resp);
}

/// Sets authentication headers for a given request.
fn sign_request(req: &mut Request, host: String) {
    // Ensure that request is a GET or HEAD to prevent signing write operations
    if ![Method::GET, Method::HEAD].contains(&req.get_method()) {
        return;
    }

    let auth = match DictionaryHandle::open("bucket_auth") {
        Ok(h) if h.is_valid() => h,
        _ => return,
    };

    let access_key_id = match auth.get("b2_application_key_id", MAX_LEN_APPLICATION_KEY_ID) {
        Ok(Some(id)) => id,
        _ => return,
    };
    let secret_access_token = match auth.get("b2_application_key", MAX_LEN_APPLICATION_KEY) {
        Ok(Some(key)) => key,
        _ => return,
    };

    // Extract region from the endpoint
    let bucket_region = REGION_REGEX.captures(host.as_str()).unwrap().get(1).unwrap().as_str().to_string();

    let client = awsv4::SignatureClient {
        access_key_id,
        secret_access_token,
        host,
        bucket_region,
        query_string: req.get_query_str().unwrap_or("").to_string()
    };

    let now = Utc::now();
    let sig = client.aws_v4_auth(req.get_method().as_str(), req.get_path(), now);

    req.set_header(header::AUTHORIZATION, sig);
    req.set_header("x-amz-content-sha256", hash("".to_string()));
    req.set_header("x-amz-date", now.format("%Y%m%dT%H%M%SZ").to_string());
}
