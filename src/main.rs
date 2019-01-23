use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::env;
use std::str::from_utf8;
use regex::Regex; // для регулярных выражений
use rand::{thread_rng, Rng};
use std::io;

fn get_hash_str() -> String {
	
	let mut initial_string = "".to_string();
	for _i in 0..5 {
		let mut rng = thread_rng();
		let f: f64 = rng.gen_range(0.0f64, 1.0f64);
		initial_string += &((6.0 * f + 1.0) as i32).to_string().chars().nth(0).unwrap().to_string();
	}
	initial_string
}

fn get_session_key() -> String {
	let mut result = "".to_string();
	for _i in 0..10{
		let mut rng = thread_rng();
		let f: f64 = rng.gen_range(0.0f64, 1.0f64);
		result += &((9.0 * f + 1.0) as i32).to_string();
	}
	result
}

fn next_session_key(key: &String, hash: &String) -> String {
	if hash == ""{
		//handle exception
		println!("ERROR: Hash string is empty");
	}
	let mut result = 0;
	for i in 0..hash.len(){ 
					//String -> char -> u64 -> i64
		result += calc_hash(&key, hash.chars().nth(i).unwrap().to_digit(10).unwrap() as i64).parse::<i64>().unwrap();
	}
	let mut temp = result.to_string();
	if temp.len() > 10 { // для обрезания пробелов, если длина < 10
		temp = temp[0..10].to_string();
	}
	let result_str = "0".repeat(10).to_string() + &temp;
	result_str[result_str.len() - 10..result_str.len()].to_string()
}

fn calc_hash(key: &String, val: i64) -> String {
	let mut result = "".to_string();
	match val {
	1 => {
		let temp = "00".to_string() + &(key[0..5].parse::<i64>().unwrap() % 97).to_string();
		return temp[temp.len() - 2..temp.len()].to_string()
		},
	2 => {
		for i in 1..key.len(){
			result += &key[key.len() - i..key.len() - i + 1];
		}
		return result.to_string() + &key.chars().nth(0).unwrap().to_string()
	},
	3 => return key[key.len() - 5..key.len()].to_string() + &key[0..5].to_string(),
	4 => {
		let mut num: i64 = 0;
		for i in 1..9{
			num += key.chars().nth(i).unwrap().to_digit(10).unwrap() as i64 + 41;
		}
		return num.to_string()
	},
	5 => {
		let mut ch:char;
		let mut num: i64 = 0;
		for i in 0..key.len(){
			ch = ((key.chars().nth(i).unwrap() as u8) ^ 43) as char;
			if !ch.is_digit(10){
				ch = (ch as u8) as char;
			}
			num += ch as i64;
		}
		return num.to_string()
	},
	_ => return (key.parse::<i64>().unwrap() + val).to_string(),
	}
}

fn handle_client(mut stream: TcpStream) {
    let mut data_1 = [0 as u8; 16]; // using 16 byte buffer
	stream.read(&mut data_1);
	let temp = from_utf8(&data_1[0..16]).unwrap().to_string();
	let split = temp.split(" ");
    let vec: Vec<&str> = split.collect();
	let hash_str = vec[0].to_string();
	let mut previous_key = vec[1].to_string();
	print!("Initial hash: {} First key: {}", hash_str, previous_key); // лог
    let mut next_key = next_session_key(&previous_key, &hash_str);
	let mut previous_key = (&next_key).to_string();
	stream.write(next_key.as_bytes()).unwrap();
	println!(" Sent key: {}", next_key); // лог
	for i in 0..4{
		let mut data_2 = [0 as u8; 10]; // using 10 byte buffer
		stream.read(&mut data_2);
		let received_key = from_utf8(&data_2[0..10]).unwrap().to_string();
		next_key = next_session_key(&previous_key, &hash_str);
		print!("Current key: {}", next_key);
		io::stdout().flush(); // Для нормальной работы print
		if received_key == next_key{
			next_key = next_session_key(&received_key, &hash_str);
			previous_key = (&next_key).to_string();
			stream.write(next_key.as_bytes()).unwrap();
			print!(" Received key: {} Status: OK Sent key: {}", received_key, next_key);
			io::stdout().flush(); // Для нормальной работы print
		}else{
			println!(" Received key: {} ERROR My current key: {}", received_key, next_key);
			break
		}
	}
}

fn start_server (port: &String) {
    let listener = TcpListener::bind("0.0.0.0:".to_string()+&port).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port {}", port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                //println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    drop(listener);
}

fn start_client(ip_port: &String) {
	// клиент
	match TcpStream::connect(&ip_port) {
        Ok(mut stream) => {	
			let mut data = [0 as u8; 16]; // буффер
			let hash_str = get_hash_str();
			let mut previous_key = get_session_key();
			println!("Initial hash: {} First key: {}", hash_str, previous_key); // лог
			stream.write(((&hash_str).to_string() + &" ".to_string() + &previous_key).as_bytes()).unwrap();
			let mut received_key = "".to_string();
			let mut next_key = "".to_string();
			for i in 0..4{
				next_key = next_session_key(&previous_key, &hash_str);
				print!("Current key: {}", next_key);
				stream.read(&mut data);
				received_key = from_utf8(&data[0..10]).unwrap().to_string();
				if received_key == next_key{
					next_key = next_session_key(&received_key, &hash_str);
					previous_key = (&next_key).to_string();
					stream.write(next_key.as_bytes()).unwrap();
					print!(" Received key: {} Status: OK Sent key: {}", received_key, next_key);
					io::stdout().flush(); // Для нормальной работы print
				}else{
					println!(" Received key: {} ERROR My current key: {}", received_key, next_key);
					break
				}
			}
			// для 10 шага (прием и сравнение без отправки
			next_key = next_session_key(&previous_key, &hash_str);
			print!("Current key: {}", next_key);
			stream.read(&mut data);
			received_key = from_utf8(&data[0..10]).unwrap().to_string();
			if received_key == next_key {
				println!(" Received key: {} Status: OK ", received_key);
			}else{
				println!(" Received key: {} ERROR My current key: {}", received_key, next_key);
			}
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}

fn main() {
	// регулярное выражение для порта
	let port_regexp = Regex::new(r"^(([0-9]{1,4})|([1-5][0-9]{4})|(6[0-4][0-9]{3})|(65[0-4][0-9]{2})|(655[0-2][0-9])|(6553[0-5]))$").unwrap();
	// регулярное выражения для ip:port
	let ip_port_regexp = Regex::new(r"^([01]?\d\d?|2[0-4]\d|25[0-5])\.([01]?\d\d?|2[0-4]\d|25[0-5])\.([01]?\d\d?|2[0-4]\d|25[0-5])\.([01]?\d\d?|2[0-4]\d|25[0-5]):((6553[0-5])|(655[0-2][0-9])|(65[0-4][0-9]{2})|(6[0-4][0-9]{3})|([1-5][0-9]{4})|([0-5]{0,5})|([0-9]{1,4}))$").unwrap();
	let args: Vec<String> = env::args().collect();
	match args.len() {
		1 => println!("lack of parameters"),
		2 => { // check port and start server
			if port_regexp.is_match(&args[1]){
				start_server(&args[1]);
			}else{
				println!("wrong port format")
			}
		},
		3 => { // check ip:port and start n clients
			if ip_port_regexp.is_match(&args[1]){
				let n = args[2].parse::<i32>().unwrap();
				for _i in 0..n {
					start_client(&args[1]);
				}
			}else{
				println!("wrong ip:port format")
			}
		},
		_ => println!("wrong number of parameters"),
	}
}
