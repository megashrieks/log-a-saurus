use crate::config::get_env;
use crate::db;
use crate::log::LogStructure;
use sqlx::{Postgres, Pool};
use tokio::fs::{File, read_dir };
use tokio::io::{AsyncReadExt, AsyncWriteExt, SeekFrom, AsyncSeekExt, BufWriter};
use tokio::time::sleep;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Duration;
use bincode::{serialize, deserialize};

pub async fn log_archiver(mut rx: tokio::sync::mpsc::Receiver<LogStructure>) {
    let current_stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
    let appendlogs = get_env("APPEND_LOGS_PATH");
    let path = format!("./{}/{}.log", appendlogs, current_stamp);
    let mut file = File::options()
        .append(true)
        .create(true)
        .open(path)
        .await
        .unwrap();

    let mut total_len: u32 = 0;
    let max_chunk_size: u32 = get_env("MAX_LOG_CHUNK_SIZE").parse::<u32>().unwrap();

    tokio::spawn(async move {
        while let Some(log) = rx.recv().await {
            let encoding = serialize(&log).unwrap();
            let encoding_length = encoding.len();
            total_len += 8 as u32 + encoding_length as u32;
            let mut writer = BufWriter::new(&mut file); // buffered writer since simultaneous
                                                        // access to file happens and we don't want
                                                        // the deserializer to read half the file
                                                        // and get an UnexpectedEof
                                                        // faced this during perf test
            writer.write_all(&(encoding_length as u64).to_le_bytes()).await.unwrap();
            writer.write_all(&encoding).await.unwrap();

            writer.flush().await.unwrap();
            if total_len >= max_chunk_size {
                total_len = 0;
                (file).shutdown().await.unwrap();
                let current_stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
                
                file = File::options()
                    .append(true)
                    .create(true)
                    .open(format!("./{}/{}.log", appendlogs, current_stamp))
                    .await
                    .unwrap();
            }
        }
    });
}


async fn get_checkpoints() -> HashMap<String, u64> {
    // get the checkpoint of the files that we've read and
    // its number of bytes read
    let appendlogs = get_env("APPEND_LOGS_PATH");
    let path = format!("./{}/checkpoint", appendlogs);
    let mut checkpoint_file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .open(path)
        .await
        .unwrap();

    let mut contents = Vec::new();
    checkpoint_file.read_to_end(&mut contents).await.unwrap();
    let dsrld = deserialize(&contents);
    (checkpoint_file).shutdown().await.unwrap();
    if let Ok(ret) = dsrld {
        ret
    } else {
        HashMap::new()
    }
}

async fn set_checkpoints(map: &HashMap<String, u64>) {
    // checkpoint whatever files we've read so far
    // and include the number of bytes read
    let appendlogs = get_env("APPEND_LOGS_PATH");
    let path = format!("./{}/checkpoint", appendlogs);
    let mut checkpoint_file = File::options()
        .write(true)
        .open(path)
        .await
        .unwrap();
    let srld = serialize(map).unwrap();
    checkpoint_file.write_all(&srld).await.unwrap();
    (checkpoint_file).shutdown().await.unwrap();
}

pub async fn get_all_logs() -> HashMap<String, u64> {
    // get a list of all log files available
    // and the sizes of the files
    let appendlogs = get_env("APPEND_LOGS_PATH");
    let mut paths = read_dir(format!("./{}", appendlogs))
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

async fn process_logs(filename: &String, seek_start: u64, pool: Pool<Postgres>, checkpoint: &mut HashMap<String, u64>) {
    let appendlogs = get_env("APPEND_LOGS_PATH");
    let path = format!("./{}/{}", appendlogs, filename);
    let mut logfile = File::options()
        .read(true)
        .open(path)
        .await
        .unwrap();
    logfile.seek(SeekFrom::Start(seek_start)).await.unwrap();
    let mut total_bytes = seek_start;
    let mut loglist: Vec<LogStructure> = Vec::new();
    loop {
        let mut log_size_bytes = [0; 8];
        match logfile.read_exact(&mut log_size_bytes).await {
            Ok(_) => {},
            Err(ref e) if e.kind() == tokio::io::ErrorKind::UnexpectedEof => break,
            Err(e) => panic!("{}", e)
        }
        let log_size = u64::from_le_bytes(log_size_bytes);
        total_bytes += 8 + log_size;
        let mut log_bytes = vec![0; log_size as usize];
        logfile.read_exact(&mut log_bytes).await.unwrap();

        let log: LogStructure = deserialize(&log_bytes).unwrap();
        // println!("Read log: {:?}", log);
        loglist.push(log);
    }
    (logfile).shutdown().await.unwrap();
    if loglist.len() > 0 {
        // inserting all logs into db and checkpointing should be an atomic transaction
        // hence it is wrapped inside a sqlx transaction
        let mut tx = pool.begin().await.unwrap();
            db::insert_logs(&mut tx, loglist).await;
            checkpoint.insert(filename.to_string(), total_bytes);
            set_checkpoints(&checkpoint).await;
        tx.commit().await.unwrap();
    }
}


pub async fn log_processor(pool: Arc<Pool<Postgres>>) {
    tokio::spawn(async move {
        loop {
            let mut checkpoint = get_checkpoints().await;
            let logfiles = get_all_logs().await;

            for (key, value) in logfiles {
                // check the logfile size against whatever we've checkpointed so far
                // if we've checkpointed less than the total size of the file, then we have things
                // left to read
                if !checkpoint.contains_key(&key) || checkpoint.get(&key).unwrap().lt(&value) {
                    let v = checkpoint.get(&key).unwrap_or(&0);
                    process_logs(&key, *v, pool.as_ref().clone(), &mut checkpoint).await;
                }
            }
            sleep(Duration::from_millis(1000)).await;
        }
    });
}
