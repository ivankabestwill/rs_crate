#![feature(box_into_raw_non_null)]

use std::ptr::NonNull;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Drop;

pub struct Cache<T>{
    value: Option<NonNull<T>>,
    info: Rc<RefCell<Info<T>>>,
}

struct Info<T>{
    depth: usize,
    list: Vec<Box<T>>,
}

pub struct CacheControl<T>{

    info: Rc<RefCell<Info<T>>>,
    f: fn()->Box<T>,
}

fn info_init<T>(depth: usize) -> Info<T>{
    Info{
        depth: depth,
        list: Vec::new(),
    }
}

pub fn cache_init<T>(depth: usize, f: fn()->Box<T>) -> CacheControl<T>{

    CacheControl{
        info: Rc::new(RefCell::new(info_init(depth))),
        f: f,
    }
}

impl <T> Drop for Cache<T>{
    fn drop(&mut self){
        match self.value{
            Some(ref t) => {
                if  self.info.borrow().depth == 0 ||
                    self.info.borrow().depth > self.info.borrow().list.len(){
                    let box_value = unsafe { Box::from_raw(t.as_ptr()) };
                    self.info.borrow_mut().list.push(box_value);
                }else{
                    unsafe{Box::from_raw(t.as_ptr())}; // here for real free T
                }
            },
            None => {},
        }

        self.value = None;
    }
}

impl <T> Cache<T>{
    pub fn get_ref(&self) -> Option<&T>{
        match self.value{
            Some(ref t) => {return Some(unsafe{t.as_ref()}); },
            None => {return None; },
        }
    }

    pub fn get_ref_mut(&mut self) -> Option<&mut T>{
        match self.value{
            Some(ref mut t) => {return Some(unsafe{t.as_mut()});},
            None => {return None;},
        }
    }
}

impl <T> CacheControl<T>{
    pub fn reset_depth(&mut self, depth: usize){
        self.info.borrow_mut().depth = depth;

        if depth > 0{
            while depth < self.info.borrow().list.len(){
                match self.info.borrow_mut().list.pop(){
                    Some(_) => {},
                    None => {break;},
                }
            }
        }
    }

    pub fn reset(&mut self){
        // clean all cache tmp
        loop {
            match self.info.borrow_mut().list.pop() {
                Some(_) => {},
                None => { break; },
            }
        }
    }

    pub fn get(&mut self) -> Cache<T>{
        let t = match self.info.borrow_mut().list.pop(){
            Some(t) => {t},
            None => {
                (self.f)()
            },
        };

        return Cache{
            info: Rc::clone(&self.info),
            value: Some(Box::into_raw_non_null(t)),
        };
    }
}

/*
use std::ops::Drop;
use std::ptr::NonNull;
use std::boxed::Box;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Cache<T>{
    data: Option<NonNull<T>>,
    cc: Rc<RefCell<CacheControl<T>>>,
}

impl <T> Drop for Cache<T>{
    fn drop(&mut self){
        let tmp_box = match self.data{
            Some(ref t) => {
                unsafe {Box::from_raw(t.as_ptr())}
            },
            None => {return;},
        };
        self.cc.borrow_mut().put(tmp_box);
        self.data = None;
    }
}

pub struct CacheControl<T>{
    depth: usize,
    caches: Vec<Box<T>>,
}

impl <T>CacheControl<T>{
    fn reset_depth(&mut self, depth: usize){
        self.depth = depth;
    }

    fn put(&mut self, t: Box<T>){
        if self.caches.len() < self.depth{
            //println!("{} {}", self.caches.len(), self.depth);
            self.caches.push(t);
        }
    }

    fn get(&mut self) -> Option<Box<T>>{
        return self.caches.pop();
    }

}
pub fn init_cache<T>(depth: usize) -> Rc<RefCell<CacheControl<T>>>{
    let cc = CacheControl{
        depth: depth,
        caches: Vec::new(),
    };

    Rc::new(RefCell::new(cc))
}

pub fn get_cache<T, F>(cc: &Rc<RefCell<CacheControl<T>>>, f: F) -> Cache<T>
where
    F: FnOnce() -> T,
{
    let box_tmp = match cc.borrow_mut().get(){
        Some(t) => {t},
        None => {Box::new(f())},
    };

    Cache{
        data: Some(Box::into_raw_non_null(box_tmp)),
        cc: Rc::clone(cc),
    }
}
*/
