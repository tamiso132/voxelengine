use std::{borrow::{Borrow, BorrowMut}, collections::HashMap, sync::{mpsc::{self, channel, Receiver, Sender}, Arc, Mutex}};

use lazy_static::lazy_static;

trait Loader{
    fn load(file_str: &str) -> PtrWrapper;
        
}

struct PtrWrapper(*mut ());
unsafe impl Sync for PtrWrapper {}

pub struct Queue{
    sender_save: Arc<Mutex<Sender<(String, u32, usize)>>>,
    reciever_save: Receiver<(String, u32, usize)>

}
impl Queue {
    pub fn get_sender(&self) -> Arc<Mutex<Sender<(String, u32, usize)>>> {
        todo!()
    }
}

pub struct Asset{
    data: PtrWrapper,
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

unsafe impl Sync for AssetLoader {
    
}



impl AssetLoader {
    fn new() -> Self{
    todo!()
    }
    pub fn save(file_name:&str, element_offset: u32, element_size: usize, data: Vec<u8>){
        
    }

    pub fn get<T>(file_name: &str, element_offset: u32, element_size: usize, count: usize){
        let is_mutated = ASSET_LOADER.is_mutated.lock().unwrap();
        let val = is_mutated.get(file_name);
       
        if val.is_none() || !val.unwrap(){
            // if it is not being loaded

            // now check if it already has been loaded
        }
    }
    
    pub fn load<T:Loader>(&self, file_name: &str){
        let mut data =  self.cache.get(file_name);
        let mut is_mutated_lock = self.is_mutated.lock().unwrap();
        {
            let mut is_mutated = is_mutated_lock.get(file_name);

            let mut in_progress = &false;

            if is_mutated.is_some(){
                in_progress = is_mutated.unwrap();
            }

            if *in_progress{
                // in progress in loading
                return;
            }
        }
            data.get_or_insert(&Asset{ data: PtrWrapper(std::ptr::null_mut())}); 
            is_mutated_lock.insert(file_name.to_owned(), true);
            
        let file_name = file_name.to_owned();
        tasc::blocking::task(|id: tasc::com::WorkerId|{
            let file_name = file_name;
            let data_ptr = T::load(&file_name);

            // need to send a complete job
            AssetLoader::load_complete(file_name, data_ptr);
        });
      
    }

    fn load_complete(file_str: String, data_ptr: PtrWrapper){
        ASSET_LOADER.cache.get(&file_str).insert(&Asset { data: data_ptr });
    }

    
}









