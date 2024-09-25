use std::{borrow::{Borrow, BorrowMut}, cell::UnsafeCell, collections::HashMap, mem::ManuallyDrop, sync::{mpsc::{self, channel, Receiver, Sender}, Arc, Mutex, RwLock}};

use lazy_static::lazy_static;
use tasc::{com, BlockingTaskHandle, Signal, TaskHandle};

pub trait Loader<T>{
    fn load(file_str: &str) -> ManuallyDrop<Vec<u8>>;
    fn reinterpret_bytes() -> T;
        
}


struct Queue{
    sender_save: Arc<Mutex<Sender<(String, u32, usize)>>>,
    reciever_save: Receiver<(String, u32, usize)>

}
impl Queue {
    pub fn get_sender(&self) -> Arc<Mutex<Sender<(String, u32, usize)>>> {
        todo!()
    }
}

#[derive(Default)]
pub struct Asset{
    data: Arc<RwLock<Vec<u8>>>,
}
lazy_static! {
    static ref ASSET_LOADER: AssetLoader = {
        AssetLoader::new()
    };
}





struct AssetLoader{
    cache: RwLock<HashMap<String, Asset>>, 
    is_mutated: RwLock<HashMap<String, bool>>,
    load_func: RwLock<HashMap<String, Arc<(com::ComHandle, Box<dyn Signal>)>>>,
}

unsafe impl Sync for AssetLoader {
    
}


impl AssetLoader {
    fn new() -> Self{

        Self { cache: RwLock::new(HashMap::new()), is_mutated: RwLock::new(HashMap::new()), load_func: RwLock::new(HashMap::new()) }
    }
    
    pub fn save(file_name:&str, element_offset: u32, element_size: usize, data: Vec<u8>){
        
    }


    pub fn get<T: Loader<R>, R>(file_name: &str){
        
        if Self::is_loading(file_name){
            // should join() the loading function
            todo!()             
        }
        else{
            // load it and wait for it
            Self::load::<T, R>(file_name, true);
        }
      
    }
    
    pub fn hot_reload<T:Loader<R>, R>(file_name: &str, wait_on: bool){
        // just reload from file, no saving done
        Self::load::<T, R>(file_name, wait_on);
    }

    pub fn load_resource<T:Loader<R>, R>(file_name: &str){
        Self::load::<T, R>(file_name, false);
    }

    pub fn load<T:Loader<R>, R>(file_name: &str, wait_on: bool){
        if Self::is_loading(file_name){
            return;
        }
        else{
            Self::set_is_loading(file_name);
        }

        Self::set_default_cache_value(file_name);

        let file_name2 = file_name.to_owned();
        let wait_on2 = wait_on.clone();

        let (com, signal) = tasc::blocking::task(move |id: tasc::com::WorkerId|{
            let file_name = file_name2;
            let wait_on = wait_on2;
            let mut data_ptr = T::load(&file_name);

            // need to send a complete job
            AssetLoader::load_complete(file_name, &mut data_ptr, wait_on);
        }).into_raw_handle_and_signal();

        if wait_on{
            com.wait_blocking(signal);
        }
        else{
            ASSET_LOADER.load_func.write().unwrap().insert(file_name.to_owned(), Arc::new((com, Box::new(signal))));
        }
    }


    fn load_complete(file_str: String, data_ptr: &mut ManuallyDrop<Vec<u8>>, wait_on: bool){
        let cache_read = ASSET_LOADER.cache.read().unwrap();
        let asset = cache_read.get(&file_str).unwrap();
    
        unsafe{
            *asset.data.write().unwrap() = Vec::from_raw_parts(data_ptr.as_mut_ptr(), data_ptr.len(), data_ptr.capacity());

            if wait_on{
                // remove from join function
            }
        }
        
    }

    fn is_loading(file_str: &str) -> bool{
        let mut is_mutated_read = ASSET_LOADER.is_mutated.read().unwrap();
        
        let mut is_mutated = is_mutated_read.get(file_str);

        let mut in_progress = &false;

        if is_mutated.is_some(){
            in_progress = is_mutated.unwrap();
        }
        *in_progress
    }

    fn set_is_loading(file_str: &str){
        let mut is_mutated_read = ASSET_LOADER.is_mutated.write().unwrap();
        is_mutated_read.insert(file_str.to_owned(), true);
    }

    fn set_default_cache_value(file_str: &str){
        let mut is_mutated_write = ASSET_LOADER.is_mutated.read().unwrap();
        {
            let mut data =  ASSET_LOADER.cache.write().unwrap();
            data.insert(file_str.to_owned(),Asset::default());
        }

    }

    
}









