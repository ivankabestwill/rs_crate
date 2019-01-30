#![feature(box_into_raw_non_null)]


use std::rc::Rc;
use std::cell::RefCell;
use std::ptr::NonNull;



const SLOT_LEN:u8 = 65; // abc  z
                           // ABC  Z
                           // 012  9
                           // .
                           // _
                           // -
#[derive(Clone)]
enum ConfigType{
    ConfigTypeValue(Rc<RefCell<ConfigValue>>),
    ConfigTypePoint(Rc<RefCell<ConfigPoint>>),
}

use self::ConfigType::{ConfigTypePoint,ConfigTypeValue};

struct ConfigValue{
    value: Rc<String>,
}

// seq node no means.
struct ConfigNode{
    v: u8,
    list: Option<NonNull<ConfigNode>>,
    next: Option<ConfigType>,
}

impl ConfigNode{
    fn show(&self){

        match self.list{
            Some(ref t) => {unsafe {t.as_ref().show()};},
            None => {},
        }

        match from_index(self.v){
            Some(ref t) => {print!("{}", t)},
            None => {print!("ConfigNode index err.");},
        }

        match self.next{
            Some(ref ct) => {
                match ct{
                    ConfigTypePoint(ref cp) =>{cp.borrow().show();},
                    ConfigTypeValue(ref cv) =>{cv.borrow().show();},
                }
            },
            None => {},
        }
    }
}

impl ConfigValue{
    fn show(&self){
        println!("={}", self.value);
    }
}

pub struct ConfigPoint{
    list: Option<NonNull<ConfigNode>>,
}

fn add_after_node_point(prelist: &mut ConfigNode, index: u8) -> Option<Rc<RefCell<ConfigPoint>>>{
    let (mut cn, cp) = new_config_node_point(index);
    cn.list = prelist.list;
    let box_cn = Box::new(cn);

    prelist.list = Some(Box::into_raw_non_null(box_cn));

    return Some(cp);
}

fn add_after_node_value(prelist: &mut ConfigNode, index: u8, value: &String) -> bool{
    let (mut cn, _cv) = new_config_node_value(index, value);
    cn.list = prelist.list;
    let box_cn = Box::new(cn);

    prelist.list = Some(Box::into_raw_non_null(box_cn));
    return true;
}

fn add_check_node_pre(prelist: &mut ConfigNode) -> Option<Rc<RefCell<ConfigPoint>>>{
    match prelist.list{
        Some(ref mut list) => {

            let list = unsafe { list.as_mut() };
            match list.next {
                Some(ref ct) => {
                    match ct {
                        ConfigTypePoint(ref cp) => { return Some(Rc::clone(cp)); },
                        _ => { return None; },
                    }
                },
                None => {
                    let cp = new_config_point();
                    let rc_ref_cp = Rc::new(RefCell::new(cp));
                    list.next = Some(ConfigTypePoint(Rc::clone(&rc_ref_cp)));

                    return Some(rc_ref_cp);
                },
            }
        },
        None => {return None;},
    }
}

fn add_check_node(list: &mut ConfigNode) -> Option<Rc<RefCell<ConfigPoint>>>{

    match list.next {
        Some(ref ct) => {
            match ct {
                ConfigTypePoint(ref cp) => { return Some(Rc::clone(cp)); },
                _ => { return None; },
            }
        },
        None => {
            let cp = new_config_point();
            let rc_ref_cp = Rc::new(RefCell::new(cp));
            list.next = Some(ConfigTypePoint(Rc::clone(&rc_ref_cp)));

            return Some(rc_ref_cp);
        },
    }
}

fn add_check_node_value(prelist: &mut ConfigNode, value: &String) -> bool{
    match prelist.list{
        Some(ref mut list) => {

            let list = unsafe { list.as_mut() };
            match list.next {
                Some(ref ct) => {
                    match ct {
                        ConfigTypeValue(ref cp) => { cp.borrow_mut().value = Rc::new(value.clone()); return true; },
                        _ => { return false; },
                    }
                },
                None => {
                    let cv = new_config_value(value);
                    let rc_ref_cv = Rc::new(RefCell::new(cv));
                    list.next = Some(ConfigTypeValue(Rc::clone(&rc_ref_cv)));

                    return true;
                },
            }
        },
        None => {return false;},
    }
}

impl ConfigPoint{
    pub fn show (&self){
        match self.list{
            None => {},
            Some(ref t) => {unsafe {t.as_ref().show()};},
        }
    }

    pub fn get_point(&self, _name: &String) -> Option<Rc<RefCell<ConfigPoint>>>{

        return None;
    }

    fn get_point_next(&self, index: u8) -> Option<Rc<RefCell<ConfigPoint>>>{
        //println!("get_point_next {}", index);

        let mut list = match self.list{
            Some(ref t) => { unsafe {t.as_ref()} },
            None => {return None;},
        };

        if list.v == index{
            match list.next{
                None => {return None;},
                Some(ref t) => {
                    match t{
                        ConfigTypePoint(ref cp) => {return Some(Rc::clone(cp));},
                        ConfigTypeValue(ref _cv) => {return None;},
                    }
                },
            }
        }

        while let Some(ref node) = list.list{

            let node_list = unsafe {node.as_ref()};

            if node_list.v == index{
                match node_list.next{
                    None => {return None;},
                    Some(ref t) => {
                        match t{
                            ConfigTypePoint(ref cp) => {return Some(Rc::clone(cp));},
                            ConfigTypeValue(ref _cv) => {return None;},
                        }
                    },
                }
            }

            list = node_list;
        }

        return None;
    }

    fn put_point(&mut self, index: u8) -> Option<Rc<RefCell<ConfigPoint>>>{
        //println!("put_point {}", index);

        let list = match self.list{
            Some(ref mut t) => {unsafe { t.as_mut() } },
            None => {
                let (cn, cp) = new_config_node_point(index);
                let box_cn = Box::new(cn);
                self.list = Some(Box::into_raw_non_null(box_cn));

                return Some(cp);
            },
        };

        if index == list.v{
            return add_check_node(list);
        } else if index < list.v{
            let (mut cn, cp) = new_config_node_point(index);
            let tmp_node = match self.list{
                Some(t) => {t},
                None => {println!("self.list is None. something err. shouldn't run here.");return None;},
            };
            cn.list = Some(tmp_node);
            let box_cn = Box::new(cn);
            self.list = Some(Box::into_raw_non_null(box_cn));

            return Some(cp);
        }

        let mut prelist = list;

        loop{

            let v = match prelist.list{
                Some(ref mut t) => {unsafe {t.as_mut().v}},
                None => {
                    return add_after_node_point(prelist, index);
                },
            };

            if index == v{
                return add_check_node_pre(prelist);
            } else if index < v{
                return add_after_node_point(prelist, index);
            }

            prelist = match prelist.list{
                Some(ref mut t) => { unsafe {t.as_mut()} },
                None => {return None;},
            };
        }
    }


    fn get_value(&self, index: u8) -> Option<Rc<String>>{
        //println!("get_value index{}", index);

        let mut list = match self.list{
            Some(ref t) => {unsafe {t.as_ref()}},
            None => {return None;},
        };

        if list.v == index{
            match list.next{
                None => {return None;},
                Some(ref t) => {
                    match t{
                        ConfigTypePoint(_) => {return None;},
                        ConfigTypeValue(ref cv) => {return Some(Rc::clone(&cv.borrow().value));},
                    }
                },
            }
        }

        while let Some(ref node) = list.list{
            let node_list = unsafe {node.as_ref()};

            if node_list.v == index{
                match node_list.next{
                    None => {return None;},
                    Some(ref t) => {
                        match t{
                            ConfigTypePoint(_) => {return None;},
                            ConfigTypeValue(ref cv) => {return Some(Rc::clone(&cv.borrow().value));},
                        }
                    },
                }
            }

            list = node_list;
        }

        return None;
    }

    fn put_value(&mut self, index: u8, value: &String) -> bool{
        //println!("put_value {} {}", index, value);

        let list = match self.list{
            Some(ref mut t) => {unsafe { t.as_mut() } },
            None => {
                let (cn, _cv) = new_config_node_value(index, value);
                let box_cn = Box::new(cn);
                self.list = Some(Box::into_raw_non_null(box_cn));

                return true;
            },
        };

        if index == list.v{
            return add_check_node_value(list, value);
        } else if index < list.v{
            let (mut cn, _cv) = new_config_node_value(index, value);
            let tmp_node = match self.list{
                Some(t) => {t},
                None => {
                    println!("put_value err: self.list cannot be None. shouldn't run here.");
                    return false;
                },
            };
            cn.list = Some(tmp_node);
            let box_cn = Box::new(cn);
            self.list = Some(Box::into_raw_non_null(box_cn));

            return true;
        }

        let mut prelist = list;

        loop{

            let v = match prelist.list{
                Some(ref mut t) => {unsafe {t.as_mut().v}},
                None => {
                    return add_after_node_value(prelist, index, value);
                },
            };

            if index == v{
                return add_check_node_value(prelist, value);
            } else if index < v{
                return add_after_node_value(prelist, index, value);
            }

            prelist = match prelist.list{
                Some(ref mut t) => { unsafe {t.as_mut()} },
                None => { return add_after_node_value(prelist, index, value); },
            };
        }
    }

    pub fn get(&self, name: &String) -> Option<Rc<String>>{
        if !name.is_ascii(){
            return None;
        }

        let mut name_bytes = name.as_bytes();

        if name_bytes.len() == 1{
            return self.get_value(name_bytes[0]);
        }

        let index = match get_index(name_bytes[0]){
            Some(t) => {t},
            None => {return None;},
        };

        let mut list_config_point = self.get_point_next(index as u8);

        while let Some(cp) = list_config_point{
            name_bytes = &name_bytes[1..];

            if name_bytes.len() == 1{
                let index = match get_index(name_bytes[0]){
                    Some(t) => {t},
                    None => {return None;},
                };
                return cp.borrow().get_value(index);
            }

            let index = match get_index(name_bytes[0]){
                Some(t) => {t},
                None => {return None;},
            };
            list_config_point = cp.borrow().get_point_next(index);
        }

        return None;
    }

    fn _put(&mut self, name: &[u8], value: &String) -> bool{

        //println!("_put {:?}", name);
        if name.len() <= 0{
            println!("_put err: name.len is <= 0");
            return false;
        }

        let index = match get_index(name[0]){
            Some(t) => {t},
            None =>{
                println!("_put err: get_index {} err.", name[0]);
                return false;
            },
        };

        if name.len() == 1{
            return self.put_value(index, value);
        }

        match self.put_point(index){
            Some(t) => {
                //println!("put_point ok.");
                return t.borrow_mut()._put(&name[1..], value);
            },
            None => {
                //println!("put point for index {} err", index);
                return false;
            },
        }
    }

    pub fn put(&mut self, name: &String, value: &String) -> bool{
        if !name.is_ascii(){
            println!("put err: name of String is not ascii.");
            return false;
        }

        let name_bytes = name.as_bytes();
        return self._put(name_bytes, value);
    }
}

#[allow(dead_code)]
fn from_index(index: u8) -> Option<char>{

    if index <= 25{
        return Some((b'a' + index) as char);
    }else if index >= 26 && index <= 51{
        return Some((b'A' + index) as char);
    }else if index == (SLOT_LEN -3) {
        return Some('.');
    }else if index == (SLOT_LEN -2){
        return Some('_');
    }else if index == (SLOT_LEN - 1){
        return Some('-');
    }

    return None;
}

fn get_index(s: u8) -> Option<u8>{
    if b'a'<=s && s<=b'z'{
        return Some(s-b'a');
    }else if b'A'<=s && s<=b'Z'{
        return Some((s-b'A')  + 26);
    }else if b'0'<=s && s<=b'9'{
        return Some((s-b'0')  + 52);
    }else if s==b'.'{
        return Some(SLOT_LEN - 3);
    }else if s==b'_'{
        return Some(SLOT_LEN - 2);
    }else if s==b'-'{
        return Some(SLOT_LEN - 1);
    }else{
        return None;
    }
}

fn new_config_value(value: &String) -> ConfigValue{
    ConfigValue{
        value: Rc::new(value.clone()),
    }
}

fn new_config_node_point(index: u8) -> (ConfigNode, Rc<RefCell<ConfigPoint>>){
    let cp = new_config_point();
    let rc_ref_cp = Rc::new(RefCell::new(cp));

    let cn = ConfigNode{
        v: index,
        next: Some(ConfigTypePoint(Rc::clone(&rc_ref_cp))),
        list: None,
    };

    return (cn, rc_ref_cp);
}

fn new_config_node_value(index: u8, value: &String) -> (ConfigNode, Rc<RefCell<ConfigValue>>){
    let cv = new_config_value(value);
    let rc_ref_cv = Rc::new(RefCell::new(cv));

    let cn = ConfigNode{
        v: index,
        list: None,
        next: Some(ConfigTypeValue(Rc::clone(&rc_ref_cv))),
    };

    return (cn, rc_ref_cv);
}

pub fn new_config_point() -> ConfigPoint{
    //println!("new_config_point");

    let cp = ConfigPoint{
        list: None,
    };

    return cp;
}








