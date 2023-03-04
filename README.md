# Fastly Compute@Edge App for Backblaze B2

[![Deploy to Fastly](https://deploy.edgecompute.app/button)](https://deploy.edgecompute.app/deploy)

Provide access to one or more private [Backblaze B2](https://www.backblaze.com/b2/cloud-storage.html) buckets via a [Fastly Compute@Edge](https://www.fastly.com/products/edge-compute) App, so that objects in the bucket may only be publicly accessed via Fastly. The app must be configured with a Backblaze application key with access to the buckets you wish to expose.

Informal testing suggests that there is negligible performance overhead imposed by signing the request.

## Configuration

In production, you must configure the app via a pair of [Edge Dictionaries](https://docs.fastly.com/en/guides/about-edge-dictionaries). You can also run the app using the local test server (`fastly compute serve`), in which case configuration is stored in `fastly.toml`.

### Credentials

If you are using private buckets (the default), you will need to create an [Edge Dictionary](https://docs.fastly.com/en/guides/about-edge-dictionaries) named `bucket_auth` with the following values:

* `b2_application_key_id` - B2 application key ID
* `b2_application_key` - B2 application key

If you use the 'Deploy to Fastly' button you will be prompted for these credentials and the Edge Dictionary will be created as part of the deployment process. Read the configuration prompts carefully as, at present, they are not shown in a consistent order.

### Bucket Name

Create a second Edge Dictionary, named `config` with the following values:

* `endpoint` - Your S3 endpoint - e.g. `s3.us-west-001.backblazeb2.com`
* `bucket_name` - See below
* `allow_list_bucket` - `true`, if you want to allow clients to list objects, otherwise `false`

Set `bucket_name` to:

* A Backblaze B2 bucket name, such as `my-bucket`, to direct all incoming requests to the specified bucket.
* `$path` to use the initial segment in the incoming URL path as the bucket name, e.g. `https://my.domain.com/my-bucket/path/to/file.png`
* `$host` to use the initial subdomain in the incoming URL hostname as the bucket name, e.g. `https://my-bucket.my.domain.com/path/to/file.png`

## Fastly CLI

You can use this repository as a template for your own app using the [`fastly` CLI](https://developer.fastly.com/reference/cli/):

```bash
fastly compute init --from=https://github.com/backblaze-b2-samples/fastly-b2-rust
```

## Acknowledgements

This demo is adapted from the [Compute@Edge static content starter kit for Rust](https://github.com/fastly/compute-starter-kit-rust-static-content).