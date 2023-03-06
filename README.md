# Fastly Compute@Edge App for Backblaze B2

[![Deploy to Fastly](https://deploy.edgecompute.app/button)](https://deploy.edgecompute.app/deploy)

This [Fastly Compute@Edge](https://www.fastly.com/products/edge-compute) App provides access to one or more private [Backblaze B2](https://www.backblaze.com/b2/cloud-storage.html) buckets, so that objects in the bucket may only be publicly accessed via Fastly. The app must be configured with a Backblaze application key with access to the buckets you wish to expose.

Informal testing suggests that there is negligible performance overhead imposed by signing the request.

## Configuration

The app can be configured to either serve data from a single named bucket or parse the bucket name from the client request. Set `bucket_name` to one of the following values:

* A Backblaze B2 bucket name, for example `my-bucket`, to direct all requests to the specified bucket. Incoming URLs do not include the bucket name. For example, `https://images.example.com/path/to/object.png`. 
* `$path` - use the initial segment in the incoming URL path as the bucket name. For example, `https://images.example.com/my-bucket/path/to/object.png`.
* `$host` - use the initial subdomain of the hostname as the bucket name. For example, `https://my-bucket.images.example.com/path/to/object.png`.

Note that, for the app to serve data from multiple buckets, the application key must be configured to access _all_ buckets in the Backblaze account.

You must configure `allowed_buckets` with a comma-separated list of buckets which clients may access, for example, `my-image-bucket,my-video-bucket,my-static-bucket`. If `bucket-name` contains the name of a bucket, then `allowed_buckets` must contain that same value for access to be allowed. 

Backblaze B2 does not allow clients to list the contents of buckets with `public_read` visibility. By default, the app follows this behaviour, but you can override this to allow clients to list buckets contents via `allow_list_bucket`.

### Development

You can run the app using the local test server (`fastly compute serve`), in which case configuration is stored in `fastly.toml`:

```toml
# Edit the backends below to match your B2 bucket and its endpoint
[local_server]
  [local_server.backends]
    [local_server.backends.b2_origin]
      # URL for B2 bucket endpoint
      url = "https://<your B2 Bucket's S3 endpoint>"

  [local_server.dictionaries]
    [local_server.dictionaries.config]
      format = "inline-toml"
      [local_server.dictionaries.config.contents]
        "allow_list_bucket" = "<true, if you want to allow clients to list objects, otherwise false>"
        "bucket_name" = "<a bucket name, $host or $path (see above)>"
        "allowed_buckets" = "<a comma-separated list of buckets to which the client is allowed access>"
        "endpoint" = "<your B2 Bucket's S3 endpoint>"
    [local_server.dictionaries.bucket_auth]
      # Credentials are stored separately
      file = "bucket_auth.json"
      format = "json"
```

Copy the file `bucket_auth.json.example` to `bucket_auth.json` and edit it to add your application key and its ID. For example:

```json
{
  "b2_application_key_id": "001samplevalue10000000001",
  "b2_application_key": "K004FsTCsamplevalueXN3i890NKm2U"
}
```

### Production

In production, you must configure the app via a pair of [Edge Dictionaries](https://docs.fastly.com/en/guides/about-edge-dictionaries).

The `bucket_auth` Edge Dictionary must contain the following values:

* `b2_application_key_id` - B2 application key ID.
* `b2_application_key` - B2 application key.

A second Edge Dictionary, named `config`, must contain the following values:

* `endpoint` - Your B2 Bucket's S3 endpoint - e.g. `s3.us-west-001.backblazeb2.com`.
* `bucket_name` - A bucket name, `$host` or `$path` (see above)
* `allowed_buckets` - A comma-separated list of buckets to which the client is allowed access.
* `allow_list_bucket` - `true`, if you want to allow clients to list objects, otherwise `false`.

If you use the 'Deploy to Fastly' button you will be prompted for these values and the Edge Dictionaries will be created as part of the deployment process. Read the configuration prompts carefully as, at present, they are not shown in a consistent order.

## Fastly CLI

You can use this repository as a template for your own app using the [`fastly` CLI](https://developer.fastly.com/reference/cli/):

```bash
fastly compute init --from=https://github.com/backblaze-b2-samples/fastly-b2-rust
```

## License

This project is licensed under the [MIT license](LICENSE).

## Acknowledgements

This demo is adapted from the [Compute@Edge static content starter kit for Rust](https://github.com/fastly/compute-starter-kit-rust-static-content).