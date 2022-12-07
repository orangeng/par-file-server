use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock, Mutex, MutexGuard};
use std::path::PathBuf;
use std::fs::File;

// FileLock keeps track of which threads 
pub struct FileLock {
    pub threads_accessing: i32,
    pub lock: Arc<RwLock<File>>,
}

// File System Reader Writer Mutex controls access to the dict keeping track of which files are currently being accessed
// and the corresponding rwlock to a file.
pub struct FsrwMutex {
    pub file_dict: Mutex<HashMap<PathBuf,FileLock>>,
}

impl FsrwMutex {
    pub fn new () -> Self {
        return Self {file_dict: Mutex::new(HashMap::new())};
    }
}

// If the file_path exists, return a rwlock pointing to that File.
// Otherwise, create that File and return a rwlock pointing to it.
pub fn acquire_file_rwlock<'a>(mut file_dict: MutexGuard<HashMap<PathBuf,FileLock>>, file_path: PathBuf) -> Arc<RwLock<File>> {
    match file_dict.get_mut(&file_path) {
        Some(file_lock) => {
            file_lock.threads_accessing += 1;
            return Arc::clone(&file_lock.lock);
        },
        None => {
            // If no file_path exists, create a file
            if !file_path.is_file() {
               File::create(&file_path).expect("acquire_file_rwlock was provided with an invalid file_path and was unable to create a file!");
            };
            
            // Open file_path with read and write access
            let file = File::options().read(true).write(true).open(file_path.clone()).unwrap();
            let rwlock = RwLock::new(file);
            let file_lock = FileLock{threads_accessing: 1,lock: Arc::new(rwlock)};
            file_dict.insert(file_path.clone(), file_lock);
            return Arc::clone(&file_dict.get(&file_path).unwrap().lock);
        }
    }
}

pub fn release_file_rwlock(mut file_dict: MutexGuard<HashMap<PathBuf,FileLock>>, file_path: PathBuf) {
    match file_dict.get_mut(&file_path) {
        Some(file_lock) => {
            file_lock.threads_accessing -= 1;
            if file_lock.threads_accessing == 0 {
                println!("{}",Arc::strong_count(&file_lock.lock));
                assert!(Arc::strong_count(&file_lock.lock) == 1);
                file_dict.remove(&file_path);
            };
        },
        None => {
            panic!("Concurrency error: thread is holding on to an invalid rwlock. file_path entry in file_dict has already been removed.");
        }
    }

}
