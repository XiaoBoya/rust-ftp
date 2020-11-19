use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::net;
use std::result::Result;
use std::str;
use regex::Regex;
use std::borrow::Borrow;
use super::status;

pub struct FTP {
    pub conn: net::TcpStream,
    pub addr: String,
}

pub fn connect(addr: &str) -> Result<FTP, Error> {
    let conn = net::TcpStream::connect(addr);
    if !conn.is_ok() {
        return Err(conn.unwrap_err());
    }
    let result = FTP {
        conn: conn.ok().unwrap(),
        addr: String::from(addr),
    };
    result.receive();
    return Ok(result);
}

impl FTP {
    // pub fn read_discard(&mut self) {
    //     self.conn.flush();
    // }
    pub fn close(&self) {
        &self.conn.shutdown(net::Shutdown::Both);
    }
    fn parse_line(&self) {} // no finished
    pub fn walk(&self) {} // no finished
    pub fn quit(&self) {} // no finished
    pub fn noop(&self) {} // no finished
    pub fn raw_cmd(&self) {} // no finished
    fn cmd(&self, expects: String, command: String) -> Result<String, Error> {
        let send_result = self.send(command);
        if send_result.is_err() {
            return Err(send_result.err().unwrap());
        }
        let line = self.receive();
        if line.is_err() {
            return line;
        }
        let content = line.unwrap();
        if !content.starts_with(&expects) {
            return Err(Error::new(ErrorKind::NotFound, content));
        }
        return Ok(content);
    }
    pub fn rename(&self, from: String, to: String) -> Result<String, Error> {
        let rename_from_command = format!("RNFR {}", from);
        let result1 = self.cmd(super::status::STATUS_ACTION_PENDING.to_string(), rename_from_command);
        if result1.is_err() {
            return Err(result1.err().unwrap())
        }
        let rename_to_command = format!("RNTO {}",to);
        let result2 = self.cmd(super::status::STATUS_ACTION_PENDING.to_string(), rename_to_command);
        return result2
    }
    pub fn mkd(&self, path: String) -> Result<String, Error> {
        let commond = format!("MKD {}", path);
        let result = self.cmd(super::status::STATUS_PATH_CREATED.to_string(), commond);
        return result;
    }
    pub fn rmd(&self, path: String) -> Result<String, Error> {
        let commond = format!("RMD {}", path);
        let result = self.cmd(super::status::STATUS_ACTION_OK.to_string(), commond);
        return result;
    }
    pub fn pwd(&self) -> Result<String,Error> {
        let result = self.cmd(super::status::STATUS_PATH_CREATED.to_string(), "PWD".to_string());
        if result.borrow().as_ref().is_err() {
            return Err(result.err().unwrap())
        }
        let path_regex = Regex::new("\"(.*)\"").unwrap();
        let content = result.borrow().as_ref().unwrap();
        let path = path_regex.captures_iter(&content.as_str()).enumerate();
        let mut r = String::new();
        for (_,v) in path {
            for i in 0..v.len() {
                r = String::from(&v[i]);
            }
        }
        return Ok(r)
    }
    pub fn cwd(&self, path:String) -> Result<String, Error> {
        let commond = format!("CWD {}", path);
        let result = self.cmd(super::status::STATUS_ACTION_OK.to_string(), commond);
        return result;
    }
    pub fn delete(&self, path:String) -> Result<String, Error> {
        let commond = format!("DELE {}", path);
        let result = self.cmd(super::status::STATUS_FILE_STATUS.to_string(), commond);
        if result.is_err() {
            return Err(result.err().unwrap())
        }
        let line = self.receive();
        if line.is_err() {
            return Err(result.err().unwrap())
        }
        let content = line.unwrap();
        if !content.starts_with("200") {
            return Err(Error::new(ErrorKind::NotFound, content));
        }
        return Ok(content);
    }
    pub fn auth_tls(&self) {} // no finished
    pub fn read_and_discard(&self) {} // no finished
    pub fn type_of(&self) {} // no finished
    fn receive_line(&self) -> Result<String, Error> {
        let mut reader = BufReader::new(&self.conn);
        let mut the_result = String::new();
        let _ = reader.read_line(&mut the_result);
        return Ok(the_result);
    }
    fn receive(&self) -> Result<String, Error> {
        let line = self.receive_line();
        if line.is_err() {
            return line;
        }
        let mut readed_line = line.ok().unwrap();
        if readed_line.len() >= 4 && &readed_line[3..4] == "-" {
            let three_str = &readed_line[..3];
            let mut closing_code = String::from(three_str);
            closing_code.push_str(" ");
            loop {
                let the_str = self.receive_line();
                if the_str.is_err() {
                    return the_str;
                }
                let content = the_str.ok().unwrap();
                readed_line.push_str(content.as_str());
                if content.len() < 4 {
                    break;
                } else {
                    if content[..4] == closing_code {
                        break;
                    }
                }
            }
        }
        return Ok(readed_line);
    }
    fn receive_no_discard(&self) {} // no finished
    fn send(&self, command: String) -> Result<usize, Error> {
        let mut conn = &self.conn;
        let mut new_command = command;
        if !new_command.ends_with("\r\n") {
            new_command.push_str("\r\n");
        }
        let result = conn.write(new_command.as_bytes());
        if result.is_err() {
            return result;
        }
        let result2 = conn.flush();
        if result2.is_err() {
            return Err(result2.err().unwrap());
        }
        return result;
    }
    pub fn pasv(&self) {} // no finished
    fn new_connection(&self) {} // no finished
    pub fn stor(&self) {} // no finished
    pub fn syst(&self) {} // no finished
    pub fn stat(&self) {} // no finished
    pub fn retr(&self) {} // no finished
    pub fn list(&self) {} // no finished
    pub fn login(&self, username: String, password: String) -> Result<String, Error> {
        let command_for_username = format!("USER {}", username);
        let result_for_username = self.cmd("331".to_string(), command_for_username);
        if result_for_username.is_err() {
            let the_error = result_for_username.err().unwrap();
            if !the_error.to_string().starts_with("230") {
                return Err(the_error);
            }
        }
        let command_for_password = format!("PASS {}", password);
        let result_for_password = self.cmd("230".to_string(), command_for_password);
        if result_for_password.is_err() {
            return Err(result_for_password.err().unwrap());
        }
        return Ok(String::new());
    }
    pub fn size(&self, path:String) -> Result<u32, Error> {
        let commond = format!("SIZE {}", path);
        let result = self.cmd(super::status::STATUS_FILE_STATUS.to_string(), commond);
        if result.is_err() {
            return Err(result.err().unwrap())
        } else {
            let mut size_mid_str = result.as_ref().unwrap();
            let size_str = &size_mid_str[4..size_mid_str.len()-2];
            let size = size_str.parse::<u32>();
            return Ok(size.ok().unwrap())
        }
    }
}
