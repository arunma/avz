use aws_sdk_s3::Client as S3Client;

use crate::error::Result;

pub async fn execute(
    files: &[String],
    s3_client: &Option<S3Client>,
    count: usize,
) -> Result<()> {
    super::cat::execute(files, s3_client, false, Some(count)).await
}
