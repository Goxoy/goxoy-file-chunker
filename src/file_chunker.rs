// #![warn(unused_assignments)]
// #![warn(unreachable_patterns)]

#[derive(Clone, Copy, Debug, PartialEq,Eq,Ord,PartialOrd)]
pub enum FileChunkType{
    Byte,
    KiloByte,
    MegaByte
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileChunk {
    pub is_exist:bool,
    pub file_name:String,
    pub file_size: u128,
    pub file_hash:String,
    pub chunk_size:u128,
    pub chunk_type:FileChunkType
}

impl FileChunk {
    pub fn new() -> Self {
        FileChunk{
            is_exist:false,
            file_name:String::new(),
            file_size:0,
            file_hash:String::new(),
            chunk_size:256,
            chunk_type:FileChunkType::KiloByte
        }
    }
    pub fn assign_file(&mut self,file_path: &str) {
        let file_meta = std::fs::metadata(file_path);
        if file_meta.is_ok(){
            self.is_exist=true;
            self.file_name=String::from(file_path);
            self.file_size=file_meta.unwrap().len() as u128;
            let bytes = std::fs::read(file_path).unwrap();
            let hash1 = blake3::hash(&bytes).to_hex();
            self.file_hash=format!("{}",hash1);
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

        //self.file
        //let bytes = std::fs::read(path).unwrap();  // Vec<u8>
        //let hash = sha256::digest_bytes(&bytes);
        //return String::new();
    }
    pub fn split(&self,_hash_data: &str) -> bool {
        //self.file
        //let bytes = std::fs::read(path).unwrap();  // Vec<u8>
        //let hash = sha256::digest_bytes(&bytes);
        return true;
    }
    pub fn merge(&self,_hash_data: &str) -> bool {
        return true;
    }
}

#[test]
fn md5_short_hash_test() {
    //let data="deneme";
    let file_path="e:/deneme.mp4";
    //let bytes = std::fs::read(file_path).unwrap();
    let mut file_obj=FileChunk::new();
    file_obj.assign_file(file_path);

    if file_obj.is_exist==true{
        file_obj.set_size(256,FileChunkType::KiloByte);
        file_obj.split("");
        dbg!(file_obj);
        println!("calisti");
    }else{
        println!("dosya yok");
    }
    assert_eq!("d8578edf8458ce06fbc5bb76a58c5ca4","d8578edf8458ce06fbc5bb76a58c5ca4")
}
