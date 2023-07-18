use serde_json::json;
use std::collections::HashMap;
use std::fs::{File, create_dir_all, self};
use std::io::{BufReader, BufRead, Write};

#[derive(Clone, Copy, Debug, PartialEq,Eq,Ord,PartialOrd)]
pub enum DefaultStoragePath{
    ExePath,
    TempPath
}
#[derive(Clone, Copy, Debug, PartialEq,Eq,Ord,PartialOrd)]
pub enum FileChunkType{
    Byte,
    KiloByte,
    MegaByte
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChunkSplitResult {
    pub file_name:String,
    pub file_hash:String,
    pub file_size: usize,
    pub chunk_size:usize,
    pub chunk_count:usize,
    pub list:HashMap<usize,String>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileChunk {
    pub is_exist:bool,
    pub file_name:String,
    pub file_size: u128,
    pub file_hash:String,
    pub chunk_size:u128,
    pub chunk_type:FileChunkType,
    pub storage_path:String,
    pub result_obj:FileChunkSplitResult,
}

impl FileChunk {
    pub fn new() -> Self {
        FileChunk{
            is_exist:false,
            file_name:String::new(),
            file_size:0,
            file_hash:String::new(),
            chunk_size:256,
            chunk_type:FileChunkType::KiloByte,
            storage_path:format!("{}", std::env::temp_dir().to_str().unwrap()),
            result_obj:FileChunkSplitResult{ 
                file_name: String::new(), 
                file_hash: String::new(), 
                file_size: 0, 
                chunk_size: 0, 
                chunk_count: 0, 
                list: HashMap::new()
            }
        }
    }
    fn calculate_file_hash(&mut self,file_path:String)->String{
        let bytes = fs::read(file_path).unwrap();
        format!("{}",blake3::hash(&bytes).to_hex())
    }
    fn control_storage_path(&self){
        _ = create_dir_all(self.storage_path.clone());
    }
    pub fn set_storage_path_with_string(&mut self, storage_path:&str) {
        self.storage_path= storage_path.to_string();
        self.control_storage_path();
    }
    pub fn set_storage_path(&mut self, which_path: DefaultStoragePath) {
        match which_path {
            DefaultStoragePath::ExePath => {
                let mut c_path=std::env::current_exe().unwrap();
                c_path.pop();
                
                c_path.pop();
                c_path.pop();
                c_path.pop();
                c_path.push("storages");
                self.storage_path=format!("{}",c_path.display());
            },
            DefaultStoragePath::TempPath => {
                self.storage_path=format!("{}", std::env::temp_dir().to_str().unwrap());
            },
        }
        self.control_storage_path();
    }
    pub fn assign_file(&mut self,file_path: &str) {
        let file_meta = fs::metadata(file_path);
        if file_meta.is_ok(){
            self.is_exist=true;
            self.file_name=String::from(file_path);
            self.file_size=file_meta.unwrap().len() as u128;
            self.file_hash=self.calculate_file_hash(file_path.to_string());
        }else{
            self.is_exist=false;
            self.file_hash=String::new();
            self.file_size=0;
            self.file_name=String::new();
            self.chunk_size=256;
            self.chunk_type=FileChunkType::KiloByte;
        }
    }
    pub fn set_size(&mut self,chunk_size:u128,chunk_type:FileChunkType) {
        self.chunk_size=chunk_size;
        self.chunk_type=chunk_type;
    }
    fn chunk_size(&self)->u128{
        match self.chunk_type{
            FileChunkType::Byte => self.chunk_size,
            FileChunkType::KiloByte => self.chunk_size*1024,
            FileChunkType::MegaByte => self.chunk_size * 1024 *1024,
        }
    }
    pub fn split(&mut self) -> bool {
        let current_chunk_size=self.chunk_size();
        if current_chunk_size<262144{
            self.chunk_type=FileChunkType::KiloByte;
            self.chunk_size=256;
        }
        let mut tmp_chunk_dir=self.storage_path.clone();
        tmp_chunk_dir.push_str("/");
        tmp_chunk_dir.push_str(&self.file_hash);
        let clear_folder_name=tmp_chunk_dir.clone();
        _ = create_dir_all(tmp_chunk_dir.clone());
        let mut split_error=false;
        let mut counter=1;
        let file = File::open(&self.file_name).unwrap();
        let mut reader = BufReader::with_capacity(self.chunk_size() as usize, file);
        let mut file_hash_list=HashMap::new();
        loop {
            let extension_str=format!("{:0>8}", counter.to_string());
            let buffer = reader.fill_buf();
            if buffer.is_ok(){
                let buffer=buffer.unwrap();
                let buffer_length = buffer.len();
                if buffer_length == 0 {
                    break;
                }
                let mut tmp_file_name=tmp_chunk_dir.clone();
                tmp_file_name.push_str("/");
                tmp_file_name.push_str("chunk.");
                tmp_file_name.push_str(&extension_str.clone());
                let create_obj = std::fs::File::create(tmp_file_name.clone());
                if create_obj.is_ok(){
                    let mut f_obj=create_obj.unwrap();
                    let write_result=f_obj.write_all(&buffer);
                    if write_result.is_err(){
                        split_error=true;
                        break;
                    }else{
                        let chunk_hash=self.calculate_file_hash(tmp_file_name.clone());
                        //println!("chunk_hash: {}",chunk_hash);
                        file_hash_list.insert(counter, chunk_hash);
                    }
                }else{
                    split_error=true;
                    break;
                }
                reader.consume(buffer_length);
            }else{
                split_error=true;
                break;
            }
            counter=counter+1;
        }
        self.result_obj=FileChunkSplitResult{ 
            file_name: String::new(), 
            file_hash: String::new(), 
            file_size: 0, 
            chunk_size: 0, 
            chunk_count: 0, 
            list: HashMap::new()
        };
        if split_error==false{
            self.result_obj.file_name=std::path::Path::new(&self.file_name).file_name().unwrap().to_str().unwrap().to_string();
            self.result_obj.file_hash=self.file_hash.clone();
            self.result_obj.file_size=self.file_size as usize;
            self.result_obj.chunk_size=self.chunk_size() as usize;
            self.result_obj.chunk_count=counter-1;
            self.result_obj.list=file_hash_list.clone();
            let mut tmp_file_name=tmp_chunk_dir.clone();
            tmp_file_name.push_str("/info.json");
            let create_obj = std::fs::File::create(tmp_file_name.clone());
            if create_obj.is_ok(){
                let mut f_obj=create_obj.unwrap();
                let info_json_obj = json!({
                    "file_name":self.result_obj.file_name,
                    "file_hash":self.result_obj.file_hash,
                    "file_size":self.result_obj.file_size,
                    "chunk_size":self.result_obj.chunk_size,
                    "chunk_count":self.result_obj.chunk_count,
                    "list":self.result_obj.list
                });
                let tmp_result_str=serde_json::to_string_pretty(&info_json_obj).unwrap();
                let tmp_write_result=f_obj.write_all(&tmp_result_str.clone().as_bytes());
                if tmp_write_result.is_ok(){
                    return true;
                }
            }
        }
        let _remove_result=fs::remove_dir_all(clear_folder_name);
        return false;
    }
    pub fn result(&self)->FileChunkSplitResult{
        self.result_obj.clone()
    }
    pub fn merge(&self,_hash_data: &str) -> bool {
        return true;
    }
}

#[test]
fn first_test() {
    // cargo test  --lib first_test -- --nocapture
    fn generate_tmp_file(file_size:usize)->String{
    let mut result_str=String::new();
    #[cfg(windows)]
    const LINE_ENDING: &str = "\r\n";
    #[cfg(not(windows))]
    const LINE_ENDING: &str = "\n";    
    let mut counter=1;
    loop{
        let micros_time=std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_micros();
        result_str.push_str("Lorem Ipsum Text Line => ");
        result_str.push_str(&counter.to_string());
        result_str.push_str(" => ");
        result_str.push_str(&micros_time.to_string());
        result_str.push_str(LINE_ENDING);
        counter=counter+1;
        if result_str.len()>file_size{
            break;
        }
    }
    let hash_file_name=format!("{}",blake3::hash(&result_str.clone().as_bytes()).to_hex());
    let mut tmp_file_name=format!("{}", std::env::temp_dir().to_str().unwrap());
    tmp_file_name.push_str(&hash_file_name);
    tmp_file_name.push_str(".txt");

    let create_obj = std::fs::File::create(tmp_file_name.clone());
    if create_obj.is_ok(){
        let mut f_obj=create_obj.unwrap();
        let write_result=f_obj.write_all(&result_str.as_bytes());
        if write_result.is_ok(){
            return tmp_file_name;
        }
    }
    return String::new();
}
    let mut error_status=true;
    let file_name=generate_tmp_file(1000000);
    let mut file_obj=FileChunk::new();
    file_obj.set_storage_path(DefaultStoragePath::TempPath);
    file_obj.assign_file(&file_name.clone());
    if file_obj.is_exist==true{
        file_obj.set_size(256,FileChunkType::KiloByte);
        let split_result=file_obj.split();
        if split_result==true{
            let _result_json=file_obj.result();
            dbg!(_result_json);
            println!("file splited");
            error_status=false;
        }
    }
    _ = fs::remove_file(file_name.clone());
    if error_status==true{
        assert_eq!(true,false)
    }else{
        assert_eq!(true,true)
    }
}
