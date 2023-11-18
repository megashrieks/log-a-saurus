use crate::log::LogStructure;
use tokio::fs::{File, read_dir};
use tokio::io::{AsyncReadExt, AsyncWriteExt, SeekFrom, AsyncSeekExt};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use bincode::{serialize, deserialize};

pub async fn log_archiver(mut rx: tokio::sync::mpsc::Receiver<LogStructure>) {
    let current_stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
    let mut file = File::options()
        .append(true)
        .create(true)
        .open(format!("./appendlogs/{}.log", current_stamp))
        .await
        .unwrap();

    let mut total_len: u32 = 0;
    const MAX_CHUNK_SIZE: u32 = 100000;

    tokio::spawn(async move {
        while let Some(log) = rx.recv().await {
            let encoding = serialize(&log).unwrap();
            let encoding_length = encoding.len();
            total_len += 8 as u32 + encoding_length as u32;
            file.write_all(&(encoding_length as u64).to_le_bytes())
                .await
                .unwrap();
            file.write_all(&encoding)
                .await
                .unwrap();

            file.flush()
                .await
                .unwrap();

            if total_len >= MAX_CHUNK_SIZE {
                let current_stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
                file = File::options()
                    .append(true)
                    .create(true)
                    .open(format!("./appendlogs/{}.log", current_stamp))
                    .await
                    .unwrap();
            }
        }
    });
}


async fn get_checkpoints() -> HashMap<String, u64> {
    let mut checkpoint_file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .open("./appendlogs/checkpoint")
        .await
        .unwrap();

    let mut contents = Vec::new();
    checkpoint_file.read_to_end(&mut contents).await.unwrap();
    let dsrld = deserialize(&contents);
    checkpoint_file.shutdown().await.unwrap();
    if let Ok(ret) = dsrld {
        ret
    } else {
        HashMap::new()
    }
}

async fn set_checkpoints(map: &HashMap<String, u64>) {
    let mut checkpoint_file = File::options()
        .write(true)
        .open("./appendlogs/checkpoint")
        .await
        .unwrap();
    let srld = serialize(map).unwrap();
    checkpoint_file.write_all(&srld).await.unwrap();
    checkpoint_file.shutdown().await.unwrap();
}

pub async fn get_all_logs() -> HashMap<String, u64> {
    let mut paths = read_dir("./appendlogs")
        .await
        .unwrap();
    let mut map:HashMap<String, u64> = HashMap::new();
    while let Some(entry) = paths.next_entry().await.unwrap() {
        let name = entry.file_name().into_string().unwrap();
        let size = entry.metadata().await.unwrap().len();
        if !name.eq("checkpoint") {
            map.insert(name, size);
        }
    }
    map
}

async fn process_logs(filename: &String, seek_start: u64) {
    let mut logfile = File::options()
        .read(true)
        .open(format!("./appendlogs/{}", filename))
        .await
        .unwrap();
    logfile.seek(SeekFrom::Start(seek_start)).await.unwrap();
    loop {
        let mut log_size_bytes = [0; 8];
        match logfile.read_exact(&mut log_size_bytes).await {
            Ok(_) => {},
            Err(ref e) if e.kind() == tokio::io::ErrorKind::UnexpectedEof => break,
            Err(e) => panic!("{}", e)
        }

        let log_size = u64::from_le_bytes(log_size_bytes);
        let mut log_bytes = vec![0; log_size as usize];
        logfile.read_exact(&mut log_bytes).await.unwrap();

        let log: LogStructure = deserialize(&log_bytes).unwrap();
        println!("Read log: {:?}", log);
    }
}
pub async fn log_processor() {
    tokio::spawn(async move {
        loop {
            let mut checkpoint = get_checkpoints().await;
            let logfiles = get_all_logs().await;

            for (key, value) in logfiles {
                if !checkpoint.contains_key(&key) || checkpoint.get(&key).unwrap().lt(&value) {
                    let v = checkpoint.get(&key).unwrap_or(&0);
                    process_logs(&key, *v).await;
                    checkpoint.insert(key, value);
                    set_checkpoints(&checkpoint).await;
                }
            }
        }
    });
}
