use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::net;
use std::result::Result;
use std::str;

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
        return Ok(String::new());
    }
    pub fn close(&self) {
        &self.conn.shutdown(net::Shutdown::Both);
    }
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
}