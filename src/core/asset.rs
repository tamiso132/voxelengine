use std::{any::Any, borrow::{Borrow, BorrowMut}, cell::UnsafeCell, collections::HashMap, mem::ManuallyDrop, sync::{mpsc::{self, channel, Receiver, Sender}, Arc, Mutex, RwLock}};

use lazy_static::lazy_static;
use tasc::{sync::SyncHandle, StdSignal};

pub trait Loader{
    type T: Clone + 'static;
    fn load(file_str: &str) -> Box<dyn Any>;
    fn save(file_str: &str, any: Box<dyn Any>);
    fn reinterpret_bytes(any: Box<dyn Any>) -> Box<Self::T>;
    fn get_full_path(file_str: &str) -> String;
        
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
struct ZeroMarker;

struct Asset{
    data: Arc<RwLock<Box<dyn Any>>>,
}

impl Asset {
}

impl Default for Asset {
    fn default() -> Self {
        Self { data: Arc::new(RwLock::new(Box::new(ZeroMarker::default()))) }
    }
}

lazy_static! {
    static ref ASSET_LOADER: AssetLoader = {
        AssetLoader::new()
    };
}





pub struct AssetLoader{
    cache: RwLock<HashMap<String, Asset>>, 
    is_mutated: RwLock<HashMap<String, bool>>,
    load_func: RwLock<HashMap<String, SyncHandle<()>>>,
}

unsafe impl Sync for AssetLoader {
    
}


impl AssetLoader {
    fn new() -> Self{

        Self { cache: RwLock::new(HashMap::new()), is_mutated: RwLock::new(HashMap::new()), load_func: RwLock::new(HashMap::new()) }
    }

    pub fn save(file_name:&str, element_offset: u32, element_size: usize, data: Vec<u8>){
        
    }


    pub fn get<loader: Loader>(file_name: &str) -> loader::T{
        let file_name = loader::get_full_path(file_name);
        if Self::is_loading(&file_name){
            let test;
          if let Some(value) = ASSET_LOADER.load_func.write().unwrap().remove(&file_name){
             test = value;
          }
          else{
            panic!("it is loading, but there is no join function");
          }

           test.wait();      
        }
        else{
            // load it and wait for it
            Self::load::<loader>(&file_name, true);
        }

        let cache = ASSET_LOADER.cache.read().unwrap();

        // Get the entry from the cache
        if let Some(entry) = cache.get(&file_name) {
            // Lock the data for reading
            let data = entry.data.read().unwrap();

            // Attempt to downcast to Box<loader::T>
            return data.downcast_ref::<loader::T>().unwrap().clone();
            
        }
        panic!("the loaded value does not exist ?????");
    }
    
    pub fn hot_reload<loader:Loader>(file_name: &str, wait_on: bool){
        // just reload from file, no saving done
        let file_name = loader::get_full_path(file_name);
        Self::load::<loader>(&file_name, wait_on);
    }

    pub fn load_resource<loader:Loader>(file_name: &str){
        let file_name = loader::get_full_path(file_name);
        Self::load::<loader>(&file_name, false);
    }

    fn load<loader:Loader>(file_name: &str, wait_on: bool){


        if Self::is_loading(&file_name){
            return;
        }
        else{
            Self::set_is_loading(&file_name);
        }

        Self::set_default_cache_value(&file_name);

        let file_name2 = file_name.to_owned();
        let wait_on2 = wait_on.clone();

        let test: SyncHandle<()> = tasc::sync::task(move ||{
            let file_name = file_name2;
            let wait_on = wait_on2;
            let mut box_any =  ManuallyDrop::new(loader::load(&file_name));

            // need to send a complete job
            AssetLoader::load_complete(file_name, box_any, wait_on);
        });

        if wait_on{
            test.wait();
        }
        else{
            ASSET_LOADER.load_func.write().unwrap().insert(file_name.to_owned(), test);
        }
    }


    fn load_complete(file_str: String, box_any: ManuallyDrop<Box<dyn Any>>, wait_on: bool){

        let cache_read = ASSET_LOADER.cache.read().unwrap();
        let asset = cache_read.get(&file_str).unwrap();
    
        unsafe{
            *asset.data.write().unwrap() = ManuallyDrop::into_inner(box_any);

            if !wait_on{
                ASSET_LOADER.load_func.write().unwrap().remove(&file_str);
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









