use super::status;
use regex::Regex;
use std::borrow::{Borrow, BorrowMut};
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::net;
use std::result::Result;
use std::str;
use std::ops::Add;

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
    fn parse_line(&self, line: String) -> (String,String,String) {
        let result_for_list = line.split(";");
        let mut perm = String::from("");
        let mut t = String::from("");
        let mut filename = String::from("");
        for v in result_for_list {
            let v2 = String::from(v).split("=");
            if v2[0] == "perm" {
                perm = String::from(v2[1]);
            } else if v2[0] == "type" {
                t = String::from(v2[1]);
            } else {
                filename = String::from(v2[1]);
            }
        }
        return (perm, t, filename)
    } // no finished
    pub fn walk(&self) {
        let mut line:Vec<String> = vec![];
    } // no finished
    pub fn quit(&self) -> Option<Error> {
        let cmd_res = self.cmd(status::STATUS_CONNECTION_CLOSING.to_string(), "QUIT".to_string());
        if cmd_res.is_err() {
            return Some(cmd_res.err().unwrap());
        }
        &self.conn.shutdown(net::Shutdown::Both);
        return None
    }
    pub fn noop(&self) -> Result<String, Error> {
        let result = self.cmd(super::status::STATUS_OK.to_string(), "NOOP".to_string());
        return result
    }
    pub fn raw_cmd(&self, commond:String) -> (i32, String) {
        let mut code = -1;

    } // no finished
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
            return Err(Error::new(ErrorKind::Other, content));
        }
        return Ok(content);
    }
    pub fn rename(&self, from: String, to: String) -> Result<String, Error> {
        let rename_from_command = format!("RNFR {}", from);
        let result1 = self.cmd(
            super::status::STATUS_ACTION_PENDING.to_string(),
            rename_from_command,
        );
        if result1.is_err() {
            return Err(result1.err().unwrap());
        }
        let rename_to_command = format!("RNTO {}", to);
        let result2 = self.cmd(
            super::status::STATUS_ACTION_PENDING.to_string(),
            rename_to_command,
        );
        return result2;
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
    pub fn pwd(&self) -> Result<String, Error> {
        let result = self.cmd(
            super::status::STATUS_PATH_CREATED.to_string(),
            "PWD".to_string(),
        );
        if result.borrow().as_ref().is_err() {
            return Err(result.err().unwrap());
        }
        let path_regex = Regex::new("\"(.*)\"").unwrap();
        let content = result.borrow().as_ref().unwrap();
        let path = path_regex.captures_iter(&content.as_str()).enumerate();
        let mut r = String::new();
        for (_, v) in path {
            for i in 0..v.len() {
                r = String::from(&v[i]);
            }
        }
        return Ok(r);
    }
    pub fn cwd(&self, path: String) -> Result<String, Error> {
        let commond = format!("CWD {}", path);
        let result = self.cmd(super::status::STATUS_ACTION_OK.to_string(), commond);
        return result;
    }
    pub fn delete(&self, path: String) -> Result<String, Error> {
        let commond = format!("DELE {}", path);
        let result = self.cmd(super::status::STATUS_OK.to_string(), commond);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let line = self.receive();
        if line.is_err() {
            return Err(result.err().unwrap());
        }
        let content = line.unwrap();
        return Ok(content);
    }
    pub fn auth_tls(&self) {} // no finished
    pub fn read_and_discard(&self) -> Result<String,Error> {
    } // no finished
    pub fn type_of(&self, t: String) -> Result<String, Error> {
        let commond = format!("TYPE {}", t);
        let result = self.cmd(super::status::STATUS_ACTION_OK.to_string(), commond);
        return result
    }
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
    fn receive_no_discard(&self) -> Result<String,Error> {
        let line_res = self.receive_line();
        if line_res.is_err() {
            return Err(line_res.err().unwrap())
        }
        let mut line = line_res.unwrap();
        if line.len() >= 4 && line[3] == "-" {
            let closing_code = &line[0..3] + " ";
            loop {
                let str_res = self.receive_line();
                if str_res.is_err(){
                    return Err(str_res.err().unwrap())
                }
                let str = str_res.unwrap();
                line = line.add(str.as_str());
                if str.len() < 4 {
                    break
                } else {
                    if str[..4] == closing_code {
                        break
                    }
                }
            }
        }
        return Ok(String::from(line))
    }
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
    pub fn syst(&self) -> Result<String, Error> {
        let res = self.send(String::from("SYST"));
        if res.is_err() {
            return Err(res.err().unwrap())
        }
        let line_res = self.receive();
        if line_res.is_err() {
            return Err(line_res.err().unwrap())
        }
        let line = line_res.unwrap();
        if !line.starts_with(status::STATUS_SYSTEM_TYPE) {
            return Err(Error::new(ErrorKind::Other, line))
        }
        let the_list = line.splitn(2, " ");
        return Ok(the_list[1])
    }
    pub fn stat(&self, path: String) -> Result<Vec<String>, Error> {
        let commond = format!("STAT {}", path);
        let result = self.cmd(super::status::STATUS_FILE_STATUS.to_string(), commond);
        if result.is_err() {
            return Err(result.err().unwrap())
        }
        let stat_res = self.receive();
        if stat_res.is_err() {
            return Err(stat_res.err().unwrap())
        }
        let stat = stat_res.unwrap();
        if !stat.starts_with(status::STATUS_FILE_STATUS) &&
            !stat.starts_with(status::STATUS_DIRECTORY_STATUS) &&
            !stat.starts_with(status::STATUS_SYSTEM_STATUS) {
            return Err(Error::new(ErrorKind::Other, stat))
        }
        let mut result:Vec<String> = vec![];
        let res = stat.split("\n");
        if stat.starts_with(status::STATUS_SYSTEM_STATUS) {
            for obj in res {
                result.append(String::from(obj).borrow_mut())
            }
            return Ok(result)
        }
        for obj in res {
            if obj.starts_with(status::STATUS_FILE_STATUS) {
                continue
            }
            result.append(String::from(obj.trim()).borrow_mut())
        }
        return Ok(result)
    }
    pub fn retr(&self) {} // no finished
    pub fn list(&self, path:String) -> Result<String, Error> {
        let before_res = self.type_of(String::from(status::TYPE_ASCII));
        if before_res.is_err() {
            return Err(before_res.err().unwrap())
        }
        return Ok(String::from(""))
    } // no finished
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
    pub fn size(&self, path: String) -> Result<u32, Error> {
        let commond = format!("SIZE {}", path);
        let result = self.cmd(super::status::STATUS_FILE_STATUS.to_string(), commond);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let mut size_mid_str = result.as_ref().unwrap();
        let size_str = &size_mid_str[4..size_mid_str.len() - 2];
        let size = size_str.parse::<u32>();
        return Ok(size.ok().unwrap());
    }
}
