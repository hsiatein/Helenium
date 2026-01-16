use anyhow::Result;
use bollard::Docker;
use bollard::query_parameters::ListImagesOptions;

#[tokio::test]
async fn test_bollard() -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    println!("{:?}", docker.info().await?);
    let images = &docker
        .list_images(Some(ListImagesOptions {
            all: true,
            ..Default::default()
        }))
        .await
        .unwrap();

    for image in images {
        println!("-> {:?}", image);
    }
    Ok(())
}
