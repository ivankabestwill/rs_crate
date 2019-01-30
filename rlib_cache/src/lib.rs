#![feature(box_into_raw_non_null)]

use std::ptr::NonNull;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Drop;

pub struct Cache<T>{
    value: Option<NonNull<T>>,
    list: Rc<RefCell<Vec<Box<T>>>>,
}

pub struct CacheControl<T,F>
    where
    F: FnOnce() -> Box<T>,
{
    depth: usize,
    list: Rc<RefCell<Vec<Box<T>>>>,
    f: NonNull<F>,
}

pub fn cache_init<T, F>(depth: usize, f: F) -> CacheControl<T, F>
    where
    F: FnOnce() -> Box<T>,
{
    let bf = Box::new(f);
    CacheControl{
        depth: depth,
        list: Rc::new(RefCell::new(Vec::new())),
        f: Box::into_raw_non_null(bf),
    }
}

impl <T> Drop for Cache<T>{
    fn drop(&mut self){
        match self.value{
            Some(ref t) => {
                let box_value = unsafe { Box::from_raw(t.as_ptr()) };
                self.list.borrow_mut().push(box_value);
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
}

impl <T,F> CacheControl<T,F>
    where
    F: FnOnce() -> Box<T>,
{
    pub fn get(&mut self) -> Cache<T>{
        let t = match self.list.borrow_mut().pop(){
            Some(t) => {t},
            None => {
               let f = unsafe { Box::from_raw(self.f.as_ptr()) };
                let t = f();
                //self.f = unsafe {Box::into_raw_non_null(f)};
                t
            },
        };

        return Cache{
            list: Rc::clone(&self.list),
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
