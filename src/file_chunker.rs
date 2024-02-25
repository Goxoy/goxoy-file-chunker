use rand::Rng;
use serde::*;
use serde_json::*;
use std::collections::HashMap;
use std::fs::{self, create_dir_all, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use substring::Substring;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};
use data_encoding::Specification;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum DefaultStoragePath {
    ExePath,
    TempPath,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum FileChunkType {
    Byte,
    KiloByte,
    MegaByte,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChunkSplitResult {
    pub info_file: String,
    pub file_name: String,
    pub file_hash: String,
    pub file_size: usize,
    pub chunk_size: usize,
    pub chunk_count: usize,
    pub list: HashMap<usize, (String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChunk {
    pub is_exist: bool,
    pub file_name: String,
    pub file_size: u128,
    pub file_hash: String,
    pub chunk_size: u128,
    pub chunk_type: FileChunkType,
    pub storage_path: String,
    pub compress_active: bool,
    pub result_obj: FileChunkSplitResult,
    pub result_list: Vec<String>,
}

impl FileChunk {
    pub fn new() -> Self {
        FileChunk {
            is_exist: false,
            file_name: String::new(),
            file_size: 0,
            file_hash: String::new(),
            chunk_size: 256,
            chunk_type: FileChunkType::KiloByte,
            storage_path: String::new(),
            compress_active: false,
            result_obj: FileChunkSplitResult {
                info_file: String::new(),
                file_name: String::new(),
                file_hash: String::new(),
                file_size: 0,
                chunk_size: 0,
                chunk_count: 0,
                list: HashMap::new(),
            },
            result_list:Vec::new()
        }
    }
    #[allow(dead_code)]
    fn hex_to_base32(&self, hex_str: String) -> String {
        let decoded = hex::decode(hex_str).unwrap();
        let spec_hex = {
            let mut spec = Specification::new();
            spec.symbols.push_str("0123456789abcdefghijklmnoprstuvy");
            spec.encoding().unwrap()
        };
        spec_hex.encode(&decoded)
    }
    fn calculate_file_hash(&self, file_path: String) -> String {
        let bytes = fs::read(file_path).unwrap();
        format!("{}", blake3::hash(&bytes).to_hex())
    }
    fn control_storage_path(&self) {
        _ = create_dir_all(self.storage_path.clone());
    }
    pub fn set_compress(&mut self, active: bool) {
        self.compress_active = active;
    }
    pub fn set_storage_path_with_string(&mut self, storage_path: &str) {
        self.storage_path = storage_path.to_string();
        self.control_storage_path();
    }
    pub fn set_storage_path(&mut self, which_path: DefaultStoragePath) {
        match which_path {
            DefaultStoragePath::ExePath => {
                let mut c_path = std::env::current_exe().unwrap();
                // c_path.pop();
                c_path.pop();
                c_path.pop();
                c_path.pop();
                c_path.push("storages");
                self.storage_path = format!("{}", c_path.display());
            }
            DefaultStoragePath::TempPath => {
                self.storage_path = format!("{}", std::env::temp_dir().to_str().unwrap());
            }
        }
        self.control_storage_path();
    }
    pub fn assign_file(&mut self, file_path: &str) {
        let file_meta = fs::metadata(file_path);
        if file_meta.is_ok() {
            self.is_exist = true;
            self.file_name = String::from(file_path);
            self.file_size = file_meta.unwrap().len() as u128;
            self.file_hash = self.calculate_file_hash(file_path.to_string());
        } else {
            self.is_exist = false;
            self.file_hash = String::new();
            self.file_size = 0;
            self.file_name = String::new();
            self.chunk_size = 256;
            self.chunk_type = FileChunkType::KiloByte;
        }
    }
    pub fn set_size(&mut self, chunk_size: u128, chunk_type: FileChunkType) {
        self.chunk_size = chunk_size;
        self.chunk_type = chunk_type;
    }
    fn chunk_size(&self) -> u128 {
        match self.chunk_type {
            FileChunkType::Byte => self.chunk_size,
            FileChunkType::KiloByte => self.chunk_size * 1024,
            FileChunkType::MegaByte => self.chunk_size * 1024 * 1024,
        }
    }
    pub fn split(&mut self) -> (bool, String) {
        self.result_list.clear();
        if self.storage_path.len() == 0 {
            self.set_storage_path(DefaultStoragePath::ExePath);
        }

        let current_chunk_size = self.chunk_size();
        if current_chunk_size < 16384 {
            self.chunk_type = FileChunkType::KiloByte;
            self.chunk_size = 64;
        }
        let mut tmp_chunk_dir = self.storage_path.clone();
        tmp_chunk_dir.push(std::path::MAIN_SEPARATOR);
        tmp_chunk_dir.push_str(&self.file_hash);
        let clear_folder_name = tmp_chunk_dir.clone();
        _ = create_dir_all(tmp_chunk_dir.clone());
        let mut split_error = false;
        let mut counter = 1;
        let file = File::open(&self.file_name).unwrap();
        let mut reader = BufReader::with_capacity(self.chunk_size() as usize, file);
        let mut file_hash_list = HashMap::new();
        loop {
            let extension_str = format!("{:0>8}", counter.to_string());
            let buffer = reader.fill_buf();
            if buffer.is_ok() {
                let buffer = buffer.unwrap();
                let buffer_length = buffer.len();
                if buffer_length == 0 {
                    break;
                }
                let raw_file_name = format!("chunk.{}", extension_str.clone());

                let mut tmp_file_name = tmp_chunk_dir.clone();
                tmp_file_name.push(std::path::MAIN_SEPARATOR);
                tmp_file_name.push_str(&raw_file_name.clone());
                let create_obj = std::fs::File::create(tmp_file_name.clone());
                if create_obj.is_ok() {
                    let mut f_obj = create_obj.unwrap();
                    let write_result = f_obj.write_all(&buffer);
                    if write_result.is_err() {
                        split_error = true;
                        break;
                    } else {
                        let zip_file_name =
                            self.compress_file(tmp_file_name.clone(), String::new());
                        if zip_file_name.len() > 0 {
                            self.result_list.push(zip_file_name.clone());
                            let path = PathBuf::from(&zip_file_name.clone());
                            let filename = path.file_name().unwrap();
                            let file_name_str = String::from(filename.to_str().unwrap());
                            let chunk_hash = self.calculate_file_hash(zip_file_name.clone());
                            _ = fs::remove_file(tmp_file_name.clone());
                            file_hash_list
                                .insert(counter, (chunk_hash, String::from(file_name_str.clone().substring(0, file_name_str.len() - 4))));
                        } else {
                            split_error = true;
                            break;
                        }
                    }
                } else {
                    split_error = true;
                    break;
                }
                reader.consume(buffer_length);
            } else {
                split_error = true;
                break;
            }
            counter = counter + 1;
        }
        self.result_obj = FileChunkSplitResult {
            info_file: String::new(),
            file_name: String::new(),
            file_hash: String::new(),
            file_size: 0,
            chunk_size: 0,
            chunk_count: 0,
            list: HashMap::new(),
        };
        if split_error == false {
            self.result_obj.file_name = std::path::Path::new(&self.file_name)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            self.result_obj.file_hash = self.file_hash.clone();
            self.result_obj.file_size = self.file_size as usize;
            self.result_obj.chunk_size = self.chunk_size() as usize;
            self.result_obj.chunk_count = counter - 1;
            self.result_obj.list = file_hash_list.clone();
            let mut tmp_file_name = tmp_chunk_dir.clone();
            tmp_file_name.push(std::path::MAIN_SEPARATOR);
            tmp_file_name.push_str("info.json");
            self.result_obj.info_file = tmp_file_name.clone();
            let create_obj = std::fs::File::create(tmp_file_name.clone());
            if create_obj.is_ok() {
                let mut f_obj = create_obj.unwrap();
                let info_json_obj = json!({
                    "info_file":self.result_obj.info_file,
                    "file_name":self.result_obj.file_name,
                    "file_hash":self.result_obj.file_hash,
                    "file_size":self.result_obj.file_size,
                    "chunk_size":self.result_obj.chunk_size,
                    "chunk_count":self.result_obj.chunk_count,
                    "list":self.result_obj.list
                });
                let tmp_result_str = serde_json::to_string_pretty(&info_json_obj).unwrap();
                let tmp_write_result = f_obj.write_all(&tmp_result_str.clone().as_bytes());
                if tmp_write_result.is_ok() {
                    let zip_file_name = self.compress_file(tmp_file_name.clone(), self.result_obj.file_hash.clone());
                    _ = fs::remove_file(tmp_file_name.clone());
                    return (true, zip_file_name.clone());
                }
            }
        }
        let _remove_result = fs::remove_dir_all(clear_folder_name);
        return (false, String::new());
    }
    fn compress_file(&self, raw_file_name: String, zip_name: String) -> String {
        let path = PathBuf::from(&raw_file_name.clone());
        let dir = path.parent().unwrap();
        let zip_raw_name = if zip_name.len() == 0 {
            self.get_time(24).to_string()
        } else {
            zip_name.clone()
        };
        let zip_file_name = format!(
            "{}{}{}.zip",
            dir.to_str().unwrap(),
            std::path::MAIN_SEPARATOR,
            zip_raw_name
        );
        let file = File::create(zip_file_name.clone());
        if file.is_err() {
            return String::new();
        }
        let mut zip = ZipWriter::new(file.unwrap());
        let ss1 = zip.start_file("raw.data", FileOptions::default());
        if ss1.is_err() {
            return String::new();
        }
        let f = File::open(raw_file_name.clone());
        if f.is_err() {
            return String::new();
        }

        let mut inner_buffer = Vec::new();
        let mut f_inner = f.unwrap();
        let rst = f_inner.read_to_end(&mut inner_buffer);
        if rst.is_err() {
            return String::new();
        }
        let ss2 = zip.write_all(&inner_buffer);
        if ss2.is_err() {
            return String::new();
        }
        let ss3 = zip.finish();
        if ss3.is_err() {
            return String::new();
        }
        return zip_file_name.clone();
    }
    pub fn result(&self) -> FileChunkSplitResult {
        self.result_obj.clone()
    }
    fn get_time(&self,fixed_length:usize) -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        if fixed_length==0{
            since_the_epoch.as_millis()
        }else{
            let mut rng = rand::thread_rng();
            let mut time_str=since_the_epoch.as_millis().to_string();
            loop{
                let result=rng.gen::<u64>().to_string().chars().nth(1).unwrap();
                time_str.push(result);
                if fixed_length==time_str.len(){
                    break;
                }
            }
            time_str.parse::<u128>().unwrap()
        }
    }
    fn change_file_name(&self, full_path: &str, new_filename: &str) -> String {
        let mut store_path = Path::new(full_path).to_owned();
        store_path.set_file_name(new_filename);
        store_path.display().to_string()
    }
    fn read_from_zip_file(&self, zip_filename: &str) -> String {
        let storage_filename = self.change_file_name(zip_filename.clone(), "raw.data");
        let path = Path::new(zip_filename.clone());
        let file = File::open(&path);
        if file.is_err() {
            return "".to_string();
        }
        let file = file.unwrap();
        let archive = ZipArchive::new(file);
        if archive.is_err() {
            return "".to_string();
        }
        let mut archive = archive.unwrap();
        for i in 0..archive.len() {
            let file = archive.by_index(i);
            if file.is_err() {
                return "".to_string();
            }
            let mut file = file.unwrap();
            if file.name().eq("raw.data") {
                let mut buffer = Vec::new();
                let rst = file.read_to_end(&mut buffer);
                if rst.is_err() {
                    return "".to_string();
                }
                let save_result = fs::write(storage_filename.clone(), buffer);
                if save_result.is_ok() {
                    return storage_filename;
                }
                return "".to_string();
            }
        }
        return "".to_string();
    }
    pub fn merge(&self, info_file_path: &str) -> bool {
        let raw_data_file_name = self.change_file_name(&info_file_path.clone(), "raw.data");
        let info_filename = self.read_from_zip_file(info_file_path);
        let contents = fs::read_to_string(info_filename.clone());
        if contents.is_err() {
            return false;
        }
        let file_contents = contents.unwrap();
        let info_obj: FileChunkSplitResult = serde_json::from_str(&file_contents).unwrap();

        let output_file_name = self.change_file_name(&info_filename.clone(), &info_obj.file_name.clone());
        if output_file_name.len()>0{
            let output_file_obj = std::fs::File::create(output_file_name.clone());
            if output_file_obj.is_err() {
                return false;
            }
            let mut output_file_obj = output_file_obj.unwrap();
            for count in 1..info_obj.chunk_count + 1 {
                let item = info_obj.list.get(&count);
                if item.is_none() {
                    _ = fs::remove_file(raw_data_file_name.clone());
                    return false;
                }
                let (chunk_hash, chunk_filename) = item.unwrap();
                let zip_filename = self.change_file_name(
                    info_file_path.clone(),
                    &format!("{}.zip", chunk_filename.clone()),
                );
                let file_meta = fs::metadata(zip_filename.clone());
                if file_meta.is_err() {
                    _ = fs::remove_file(raw_data_file_name.clone());
                    return false;
                }
                if info_obj.list.contains_key(&count) == false {
                    _ = fs::remove_file(raw_data_file_name.clone());
                    return false;
                }
                let chunk_file_hash = self.calculate_file_hash(zip_filename.clone().to_string());
                if chunk_hash.eq(&chunk_file_hash.clone()) == false {
                    _ = fs::remove_file(raw_data_file_name.clone());
                    return false;
                }
                let chunk_filename = self.read_from_zip_file(&zip_filename.clone());
                if chunk_filename.len()>0{
                    let bytes_buf = fs::read(chunk_filename.clone()).unwrap();
                    let write_result = output_file_obj.write_all(&bytes_buf);
                    if write_result.is_err() {
                        _ = fs::remove_file(raw_data_file_name.clone());
                        return false;
                    }
                }
            }        
        }
        _ = fs::remove_file(raw_data_file_name.clone());
        let calculated_file_hash = self.calculate_file_hash(output_file_name.clone().to_string());
        if info_obj.file_hash.eq(&calculated_file_hash.clone()) == false {
            return false;
        }
        return true;
    }
}

#[test]
fn full_test() {
    // cargo test  --lib full_test -- --nocapture
    fn generate_tmp_file(file_size: usize) -> String {
        let mut result_str = String::new();
        #[cfg(windows)]
        const LINE_ENDING: &str = "\r\n";
        #[cfg(not(windows))]
        const LINE_ENDING: &str = "\n";
        let mut counter = 1;
        loop {
            let micros_time = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();
            result_str.push_str("Lorem Ipsum Text Line => ");
            result_str.push_str(&counter.to_string());
            result_str.push_str(" => ");
            result_str.push_str(&micros_time.to_string());
            result_str.push_str(LINE_ENDING);
            counter = counter + 1;
            if result_str.len() > file_size {
                break;
            }
        }
        let hash_file_name = format!("{}", blake3::hash(&result_str.clone().as_bytes()).to_hex());
        let mut tmp_file_name = format!("{}", std::env::temp_dir().to_str().unwrap());
        tmp_file_name.push_str(&hash_file_name);
        tmp_file_name.push_str(".txt");

        let create_obj = std::fs::File::create(tmp_file_name.clone());
        if create_obj.is_ok() {
            let mut f_obj = create_obj.unwrap();
            let write_result = f_obj.write_all(&result_str.as_bytes());
            if write_result.is_ok() {
                return tmp_file_name;
            }
        }
        return String::new();
    }
    let mut error_status = true;
    let file_name = generate_tmp_file(1000000);
    let mut file_obj = FileChunk::new();
    file_obj.set_storage_path(DefaultStoragePath::TempPath);
    file_obj.assign_file(&file_name.clone());
    if file_obj.is_exist == true {
        file_obj.set_size(256, FileChunkType::KiloByte);
        let (split_result, _archive_name) = file_obj.split();
        if split_result == true {
            let _result_json = file_obj.result();
            let merge_result = file_obj.merge(&_result_json.info_file);
            if merge_result == true {
                error_status = false;
            }

            //dizin icindeki gecici dosyalar siliniyor...
            let path = Path::new(&_result_json.info_file);
            let mut base_path = path.parent().unwrap().display().to_string();
            base_path.push(std::path::MAIN_SEPARATOR);
            for file in fs::read_dir(base_path).unwrap() {
                let clear_file_name = file.unwrap().path().display().to_string();
                _ = fs::remove_file(clear_file_name.clone());
            }
        }
    }
    // olusturulan temp dosyasi siliniyor
    _ = fs::remove_file(file_name.clone());
    if error_status == true {
        assert_eq!(true, false)
    } else {
        assert_eq!(true, true)
    }
}
