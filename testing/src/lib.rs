#[cfg(test)]
mod test {
    use std::io::Write;

    use mrklar::ServerConfig;
    use mrklar_api::MrklarApi;
    use mrklar_common::config::DEFAULT_SERVER_PORT;
    use mrklar_fs::{gen_tmp_filename, get_test_files_dir, sha256};
    use tempfile::tempdir;

    async fn start_server(config: ServerConfig) -> MrklarApi {
        let api = MrklarApi::new(config.net.clone());
        tokio::spawn(async move { mrklar::spawn(config).await });
        api
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_spawn_empty() {
        let tmp_empty_db_dir = tempdir().unwrap();
        println!("test db dir='{:?}'", tmp_empty_db_dir.path());

        let config = ServerConfig::test_default()
            .with_port(DEFAULT_SERVER_PORT)
            .with_tracing(false)
            .with_db_dir(tmp_empty_db_dir.path().to_path_buf());

        let api = start_server(config.clone()).await;

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let a = api.count().await.unwrap();
        assert_eq!(a, 0);

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let a = api.count().await.unwrap();
        assert_eq!(a, 0);

        tmp_empty_db_dir.close().unwrap();
    }

    /// Upload + Download + Verify one file
    #[tokio::test(flavor = "multi_thread")]
    async fn test_one_file() {
        let tmp_empty_db_dir = tempdir().unwrap();
        println!("test db dir='{:?}'", tmp_empty_db_dir.path());

        let tmp_empty_files_dir = tempdir().unwrap();
        println!("test files dir='{:?}'", tmp_empty_files_dir.path());

        // inc the port to avoid port conflict
        let config = ServerConfig::test_default()
            .with_port(DEFAULT_SERVER_PORT + 1)
            .with_tracing(false)
            .with_db_dir(tmp_empty_db_dir.path().to_path_buf())
            .with_files_dir(tmp_empty_files_dir.path().to_path_buf());

        let api = start_server(config.clone()).await;

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let a = api.count().await.unwrap();
        assert_eq!(a, 0);

        let p = get_test_files_dir().unwrap().join("0");
        let (file_index, merkle_root) = api.upload(&p).await.unwrap();
        let p_sha256 = sha256(p).unwrap();

        let zero = config.files_db_dir().join("0");
        assert!(zero.is_file());
        assert_eq!(sha256(zero).unwrap(), p_sha256);

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let count_files = api.count().await.unwrap();
        assert_eq!(count_files, 1);

        let merkle_proof = api.proof(file_index).await.unwrap();
        assert!(merkle_proof.verify(&p_sha256));
        assert_eq!(merkle_proof.root(), &merkle_root);

        tmp_empty_db_dir.close().unwrap();
        tmp_empty_files_dir.close().unwrap();
    }

    /// Upload + Download + Verify 300 randomly generated files
    #[tokio::test(flavor = "multi_thread")]
    async fn test_all_sequential() {
        const N_FILES: usize = 300;

        let tmp_db_dir = tempdir().unwrap();
        println!("test db dir={:?}", tmp_db_dir.path());

        let tmp_files_dir = tempdir().unwrap();
        println!("test files dir={:?}", tmp_files_dir.path());

        let tmp_src_dir = tempdir().unwrap();
        println!("test src dir={:?}", tmp_src_dir.path());
        let tmp_src_path = tmp_src_dir.path().to_path_buf();

        let tmp_dl_dir = tempdir().unwrap();
        println!("test dl dir={:?}", tmp_dl_dir.path());
        let tmp_dl_path = tmp_dl_dir.path().to_path_buf();

        // inc the port to avoid port conflict
        let config = ServerConfig::default()
            .with_port(DEFAULT_SERVER_PORT + 2)
            .with_tracing(false)
            .with_db_dir(tmp_db_dir.path().to_path_buf())
            .with_files_dir(tmp_files_dir.path().to_path_buf());

        let mut file_names = vec![];

        // 1- generate N files in src dir
        for _i in 0..N_FILES {
            let s = gen_tmp_filename();
            let p = tmp_src_path.join(&s);
            file_names.push(p);
            let mut f = std::fs::File::create(file_names.last().unwrap()).unwrap();
            f.write_all(s.as_bytes()).unwrap();
            f.sync_all().unwrap();
        }

        assert_eq!(file_names.len(), N_FILES);

        // 2- compute each file sha256 hash
        let mut file_sha256s = vec![];
        for i in 0..N_FILES {
            file_sha256s.push(sha256(&file_names[i]).unwrap());
        }

        // 3- start server
        let api = start_server(config.clone()).await;

        // 4- upload all files
        let mut file_infos = vec![];
        for i in 0..N_FILES {
            // index, merkle_root
            let info = api.upload(&file_names[i]).await.unwrap();
            assert_eq!(info.0, i as u64);
            file_infos.push(info);
        }

        // 5- make sure all files are stores
        let count = api.count().await.unwrap();
        assert_eq!(count, N_FILES as u64);

        // 6- verify the merkle root
        let root = api.root().await.unwrap();
        assert_eq!(root, file_infos.last().unwrap().1);

        // 7- compute and verify each proof
        for i in 0..N_FILES {
            // index, merkle_root
            let proof = api.proof(i as u64).await.unwrap();
            let ok = proof.verify(&file_sha256s[i]);
            assert!(ok);
        }

        // 8- download all files, compute sha, verify each proof
        for i in 0..N_FILES {
            // index, merkle_root
            let dl_result = api
                .download(
                    i as u64,
                    Some(tmp_dl_path.clone()),
                    None,
                    false,
                )
                .await
                .unwrap();
            assert!(dl_result.0.is_file());
            let expected_path = tmp_dl_path.join(file_names[i].file_name().unwrap());
            assert!(expected_path.is_file());

            let dl_sha256 = sha256(&dl_result.0).unwrap();
            let expected_sha256 = sha256(&expected_path).unwrap();

            assert_eq!(&dl_sha256, &expected_sha256);

            let ok = dl_result.1.verify(&dl_sha256);
            assert_eq!(dl_result.1.root(), &root);
            assert!(ok);
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        tmp_dl_dir.close().unwrap();
        tmp_src_dir.close().unwrap();
        tmp_db_dir.close().unwrap();
        tmp_files_dir.close().unwrap();
    }
}
