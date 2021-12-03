pub async fn sync() -> fpm::Result<()> {
    let config = fpm::Config::read().await?;

    std::fs::create_dir_all(format!("{}/.history", config.root.as_str()).as_str())
        .expect("failed to create build folder");

    let timestamp = fpm::get_timestamp_nanosecond();

    let mut modified_files = vec![];
    for doc in fpm::process_dir(config.root.as_str()).await? {
        if let Some(file) = write(&doc, timestamp).await? {
            modified_files.push(file);
        }
    }
    if modified_files.is_empty() {
        println!("Everything is upto date.");
    } else {
        println!(
            "Repo for {} is github, directly syncing with .history.",
            config.package.name
        );
        for file in modified_files {
            println!("{}", file);
        }
    }
    Ok(())
}

async fn write(doc: &fpm::Document, timestamp: u128) -> fpm::Result<Option<String>> {
    use tokio::io::AsyncWriteExt;

    if doc.id.starts_with(".history") {
        return Ok(None);
    }

    let (path, doc_name) = if doc.id.contains('/') {
        let (dir, doc_name) = doc.id.rsplit_once('/').unwrap();
        std::fs::create_dir_all(format!("{}/.history/{}", doc.base_path.as_str(), dir))?;
        (
            format!("{}/.history/{}", doc.base_path.as_str(), dir),
            doc_name.to_string(),
        )
    } else {
        (
            format!("{}/.history", doc.base_path.as_str()),
            doc.id.to_string(),
        )
    };

    let mut files = tokio::fs::read_dir(&path).await?;

    let mut max_timestamp: Option<(String, String)> = None;
    while let Some(n) = files.next_entry().await? {
        let p = format!("{}/{}.", path, doc_name.replace(".ftd", ""));
        let file = n.path().to_str().unwrap().to_string();
        if file.starts_with(&p) {
            let timestamp = file
                .replace(&format!("{}/{}.", path, doc_name.replace(".ftd", "")), "")
                .replace(".ftd", "");
            if let Some((t, _)) = &max_timestamp {
                if *t > timestamp {
                    continue;
                }
            }
            max_timestamp = Some((timestamp, file.to_string()));
        }
    }

    if let Some((_, path)) = max_timestamp {
        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if doc.document.eq(&existing_doc) {
            return Ok(None);
        }
    }

    let new_file_path = format!(
        "{}/.history/{}",
        doc.base_path.as_str(),
        doc.id.replace(".ftd", &format!(".{}.ftd", timestamp))
    );

    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;

    f.write_all(doc.document.as_bytes()).await?;
    Ok(Some(doc.id.to_string()))
}
