use std::{collections::HashMap, sync::{mpsc::{self, channel, Receiver, Sender}, Arc, Mutex}};

use lazy_static::lazy_static;

pub struct Queue{
    sender_save: Arc<Mutex<Sender<(String, u32, usize)>>>,
    reciever_save: Receiver<(String, u32, usize)>

}
}
impl Queue {
    pub 
    pub fn get_sender(&self) -> Arc<Mutex<Sender<(String, u32, usize)>>> {
        self.sender.clone()
    }
}

pub struct Asset{
    is_loading: bool,
    data: Vec<u8>,
}

lazy_static! {
    static ref ASSET_LOADER: AssetLoader = {
        AssetLoader::new()
    };
}

pub struct AssetLoader{
    cache: HashMap<String, Asset>,
    is_mutated: Mutex<HashMap<String, bool>>,
}



impl AssetLoader {
    fn new() -> Self{
    todo!()
    }
    pub fn save(file_name:&str, element_offset: u32, element_size: usize, data: Vec<u8>){
        
    }

    pub fn get<T>(file_name: &str, element_offset: u32, element_size: usize, count: usize){

    }
    
    pub fn load(&self, file_name: &str){
        let data =  self.cache.get(file_name);
        {
            let mut is_mutated_lock = self.is_mutated.lock().unwrap();
            let mut is_mutated = is_mutated_lock.get_mut(file_name);

            let mut in_progress = &mut false;

            if is_mutated.is_some(){
                in_progress = is_mutated.unwrap();
            }

            if *in_progress{
                // in progress in loading
                return;
            }
            else{
                // start loading
            }

        }
      
    }

    
}


fn load_file(file_name: &str){

}






