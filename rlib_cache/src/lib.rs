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
    stat: Stat,
    depth: usize,
    list: Vec<Box<T>>,
}

struct Stat{
    alloc_num: usize,
    reuse_num: usize,
    free_num: usize,
    recyle_num: usize,
}

pub struct CacheControl<T>{
    info: Rc<RefCell<Info<T>>>,
    f: fn()->Box<T>,
}

fn info_init<T>(depth: usize) -> Info<T>{
    Info{
        stat: stat_init(),
        depth: depth,
        list: Vec::new(),
    }
}

fn stat_init() -> Stat{
    Stat{
        alloc_num: 0,
        reuse_num: 0,
        free_num: 0,
        recyle_num: 0,
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
                    self.info.borrow_mut().stat.recyle_num += 1;
                }else{
                    self.info.borrow_mut().stat.free_num += 1;
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
            Some(t) => {
                self.info.borrow_mut().stat.reuse_num += 1;
                t
            },
            None => {
                self.info.borrow_mut().stat.alloc_num += 1;
                (self.f)()
            },
        };

        return Cache{
            info: Rc::clone(&self.info),
            value: Some(Box::into_raw_non_null(t)),
        };
    }
}
