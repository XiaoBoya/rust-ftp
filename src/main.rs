use std::borrow::{Borrow};

mod ftp;
mod status;

fn main() {
    let result = ftp::connect("172.26.106.28:2121");
    let the_result = result.borrow();
    if the_result.is_ok() {
        let login_result = the_result.as_ref().unwrap().login("admin".to_string(), "123456".to_string());
        if login_result.is_err() {
            println!("{}", login_result.err().unwrap().to_string())
        }
        // let mkd_result = the_result.as_ref().unwrap().mkd("hahaha".to_string());
        // if mkd_result.is_err() {
        //     println!("{}", mkd_result.err().unwrap().to_string())
        // } else {
        //     println!("{}", mkd_result.ok().unwrap())
        // }
        // let rename_result = the_result.as_ref().unwrap().rename("hahaha".to_string(),"heng".to_string());
        // if rename_result.is_err() {
        //     println!("{}", rename_result.err().unwrap().to_string())
        // } else {
        //     println!("{}", rename_result.ok().unwrap())
        // }
        // let pwd_result = the_result.as_ref().unwrap().pwd();
        // if pwd_result.is_err() {
        //     println!("{}", pwd_result.err().unwrap().to_string())
        // } else {
        //     println!("{}", pwd_result.ok().unwrap())
        // }
    }
}