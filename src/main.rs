use std::io::BufWriter;
use std::io::BufReader;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;



struct EncoderTeranslateTable{
    word_len : u8,
    word : u32
}
impl EncoderTeranslateTable {
    fn new(_word_len: u8, _word: u32, _debug_string: String) -> Self {
        EncoderTeranslateTable{
            word_len : _word_len,
            word : _word
        }
    }
}
struct DecoderTeranslateTable{
    word : u32,
    word_len : u8,
    char_value : u8
}
impl DecoderTeranslateTable {
    fn new(_char_value: u8, _word: u32, _word_len: u8) -> Self {
        DecoderTeranslateTable{
            char_value: _char_value,
            word : _word,
            word_len : _word_len
        }
    }
}

struct Node {
    count: i64,
    char_value: char,
    children: Vec<Node>
}
impl Node {
    fn new(value: char, count: i64) -> Self {
        Node {
            count: count,
            char_value: value,
            children : Vec::<Node>::new()
        }
    }
    fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }
}
struct Reader{
    buffer: [u8; 1],
    still_readable: u8,
    buffered_reader : BufReader<File>
}
impl Reader{
    pub fn new(input_file_name: &str) -> Reader {
        Reader { buffer: [0], still_readable: 0, buffered_reader :  BufReader::new(File::open(input_file_name).expect("opening file error"))}
    }
    pub fn read_bit(& mut self) -> u8{
      if self.still_readable == 0 {
        self.buffered_reader.read(&mut self.buffer).expect("reading error");
        self.still_readable = 8;
      }
      let bit = self.buffer[0] / 128;
      //println!("{}", self.buffer[0]);
      //println!("{}", self.still_readable);
      self.buffer[0] = self.buffer[0] % 128* 2;
      self.still_readable -= 1;

      if bit > 2{
          panic!("read bit value is more than 2");
      }
      return bit;
    }
    pub fn read_6bit(& mut self) -> u8{
        let mut word : u8 = 0;
        for _i in 0..6{
            let c = self.read_bit();
            word = word*2;
            word += c;
        }
        return word;
    }
    pub fn read_byte(& mut self)-> u8{
        let mut word : u8 = 0;
        for _i in 0..8{
            let c = self.read_bit();
            word = word*2;
            word += c;
        }
        return word;
    }
}
struct Writer {
    buffer: [u8; 1],
    written : u8,
    buff_writer : BufWriter<File>
}

impl Writer {
    pub fn new(output_file_name: &str) -> Writer {
        Writer { buffer: [0], written: 0, buff_writer : BufWriter::new(File::create(output_file_name).expect("Create error"))}
    }
    fn flush(& mut self){
        while self.written < 8{
            self.write_on_buff_bit(0);
        }
        self.write_on_buff_bit(0);
        self.buff_writer.flush().expect("Could not flush");
    }
    fn write_on_buff_bit(& mut self , value : u8){      
        if value > 1 {
             panic!("Do not call with > 1");
        }        
        if self.written == 8{
            self.buff_writer.write(&self.buffer).expect("impossible to write");
            //print!("unload");
            self.buffer[0] = 0;
            self.written = 0;
        }
        self.buffer[0] = self.buffer[0] % 128 * 2;
        self.buffer[0] +=  value%2;
        
        
        self.written += 1;
    }
    fn write_on_buff_6bit(&mut self, mut value : u8){
        for _i in 0..6{
            self.write_on_buff_bit(value/32);
            value = value << 1;
            value = value %64;
        }
    }

    fn write_on_buff_byte(&mut self, mut value : u8){
        for _i in 0..8{
            self.write_on_buff_bit(value/128);
            value = value << 1;
            //println!("{}", value);
        }
    }
    fn write_huff_code(&mut self,huff_word: u32, huff_len: u8){
        let mut x : u32 = (huff_len - 1) as u32;
        let mut word = huff_word;
        loop {
            let base_two: u32 = 2; // an explicit type is required
            //println!("{}",base_two);
            let this_iteration_pow = base_two.pow(x);
            //println!("{}",this_iteration_pow);
            let this_iteration = word / this_iteration_pow;
            //println!("{}",this_iteration);
            word = word % this_iteration_pow ;
            //println!("{}",word);
            self.write_on_buff_bit(this_iteration as u8);
            //println!("This iteration {}", this_iteration);
            if x == 0 {
                break;
            }
            x-=1;
        }
    }
}

fn get_huff_tree(key_value : [i64; 256]) -> Node{
    let mut not_in_tree_nodes = Vec::new();

    for i in 0 .. 256{
        if key_value[i] != 0 {
            not_in_tree_nodes.push(Node::new(i  as u8 as char, key_value[i]));
        }
    }

    //debug_show_list(&not_in_tree_nodes);

    while not_in_tree_nodes.len() > 1{
        not_in_tree_nodes.sort_by(|a, b| b.count.cmp(&a.count));
        
        let right_node = not_in_tree_nodes.remove(not_in_tree_nodes.len() - 1);
        let left_node = not_in_tree_nodes.remove(not_in_tree_nodes.len() - 1);
        let mut dad_node = Node::new(0 as char, left_node.count + right_node.count);
        dad_node.add_child(right_node);
        dad_node.add_child(left_node);

        not_in_tree_nodes.push(dad_node);

        //debug_show_list(&not_in_tree_nodes);
    }

    return not_in_tree_nodes.remove(not_in_tree_nodes.len() - 1);
}

fn navigate_tree(node : Node, debug_string: & str, word : u32, word_len: u8) -> HashMap<char,EncoderTeranslateTable> {
    let mut local_hash :HashMap<char, EncoderTeranslateTable> = HashMap::new();
    let mut index = 0;
    for n in node.children{
        let s1: String = format!("{}{}", debug_string, &index.to_string());

        let mut sub =  navigate_tree(n, &s1, (word *2) + index, word_len + 1);
        for (k, v) in sub.drain() {
            local_hash.insert(k, v);
        }
        index += 1;
    }
    if index == 0 {
        local_hash.insert(node.char_value, EncoderTeranslateTable::new(word_len, word, format!("{}", debug_string) ));
    }
    return local_hash;
}



fn write_to_file(mut writer :  Writer, hash_map : & HashMap<char,EncoderTeranslateTable>, original_file_name: &str, magic_word: [char; 7], file_size : u32){

    for c in magic_word.iter(){
        writer.write_on_buff_byte(*c as u8);
    }

    println!("table lenght {}", (hash_map.len() - 1) as u8);
    writer.write_on_buff_byte((hash_map.len() - 1) as u8);

    for (k, v) in hash_map {
        //println!("{} : debug_string {}, word: {}, word_len {}", k, v.debug_string, v.word, v.word_len);

        writer.write_on_buff_byte(*k as u8);
        writer.write_on_buff_6bit(v.word_len);
        writer.write_huff_code(v.word, v.word_len);
    }
    
    let base_two = 2 as u32;
    // println!("{}",(file_size / base_two.pow(24)) as u8);
    writer.write_on_buff_byte((file_size / base_two.pow(24)) as u8);
    // println!("{}",((file_size / base_two.pow(8)) % base_two.pow(8)) as u8);
    writer.write_on_buff_byte((file_size / base_two.pow(16) % base_two.pow(8)) as u8);
    // println!("{}",((file_size / base_two.pow(8)) % base_two.pow(8)) as u8);
    writer.write_on_buff_byte(((file_size / base_two.pow(8)) % base_two.pow(8)) as u8);
    // println!("{}",((file_size) % base_two.pow(8)) as u8);
    writer.write_on_buff_byte(((file_size) % base_two.pow(8)) as u8);
    
    let mut f = File::open(original_file_name).expect("Could not read file");
    let mut buffer : [u8; 1] = [0; 1];
    //println!("Original file size {}", file_size);
    // read up to 10 bytes
    while f.read(&mut buffer).expect("impossible to read from file") > 0{
        let key = buffer[0] as char; 
        let value = &hash_map[&key];
        writer.write_huff_code(value.word, value.word_len)
    }
    writer.flush();
}

fn huff_encode(original_file_name: &str, huff_encoded_file_name: &str, magic_word: [char; 7]){
    let mut buffered_reader =  BufReader::new(File::open(original_file_name).expect("Could not read file"));
    let mut buffer : [u8; 1] = [0; 1];
    let mut file_size : u32 = 0;
    let mut key_value : [i64; 256] = [0; 256];
    // read up to 10 bytes
    while buffered_reader.read(&mut buffer).expect("impossible to read from file") > 0{
        file_size += 1;
        key_value[buffer[0] as usize] += 1;
        if key_value[buffer[0] as usize] >= i64::MAX{
            panic!("Too many repetition of char {}", buffer[0]);
        } 
    }
    println!("{}", file_size);
    let tree_root = get_huff_tree(key_value);

    let hash_map = navigate_tree(tree_root, &String::from(""), 0, 0);
    
    println!("ENCODER HASH MAP");
    // for (k, v) in &hash_map {
    //     println!("{} : word: {}, word_len {}", *k as u8, v.word, v.word_len);
    // }

    let writer = Writer::new(huff_encoded_file_name);
    write_to_file(writer, &hash_map,original_file_name, magic_word, file_size);
}

fn huff_decode(huff_encoded_file_name: &str, huff_decoded_file_name: &str, magic_word: [char; 7]){
    let mut buff_output = BufWriter::new(File::create(huff_decoded_file_name).expect("Error creating file"));
    let mut reader = Reader::new(huff_encoded_file_name);
    for c in magic_word.iter(){
        let x = reader.read_byte();
        if x != *c as u8 {
            panic!("Magic Word not found, this file is not huffman encoded");
        }
    }
    let _hash_map_to_read = reader.read_byte();
    let mut  hash_map_to_read = _hash_map_to_read as u16;
    hash_map_to_read += 1;
    
    let mut hash_map : Vec<DecoderTeranslateTable> = Vec::new();

    for _i in 0 .. hash_map_to_read{

        let key : u8  = reader.read_byte();
        let huff_code_len = reader.read_6bit();

        let mut huff_code : u32 = 0;
        let mut huff_code_debug  = String::from("");
        //format!("{}{}", debug_string, &index.to_string());
        for _i in 0 .. huff_code_len {
            let readed = reader.read_bit();
            //println!("{}", readed);
            huff_code *= 2;
            huff_code += readed as u32;
            huff_code_debug = format!("{}{}", huff_code_debug, &readed.to_string());
        }

        hash_map.push(DecoderTeranslateTable::new(key as u8, huff_code, huff_code_len));
    }
    println!("DECODER HASH MAP");
    println!("hash map size {}", hash_map.len());
    // for v in &hash_map {
    //     println!("{} : word: {}", v.char_value, v.word);
    // }
    let mut word32 : u32 = 0;
    let base_two : u32 = 2;
    word32 += reader.read_byte() as u32;
    //println!("{}", word32);
    word32 *= base_two.pow(8);
    word32 += reader.read_byte() as u32;
    //println!("{}", word32);
    word32 *= base_two.pow(8);
    word32 += reader.read_byte() as u32;
    //println!("{}", word32);
    word32 *= base_two.pow(8);
    word32 += reader.read_byte() as u32;

    println!("readed ->{}", &word32);
    // println!("");
    for _i in 0 .. word32{
        let mut current_word : u32 = 0;
        let mut loop_counter = 0;
        let mut double_break = false;
        loop{
            let readed = reader.read_bit();
            //println!("{}", readed);
            current_word *= 2;
            current_word += readed as u32;
            loop_counter += 1;
            //println!("{}", current_word);
            for val in &hash_map{
                if val.word == current_word && val.word_len == loop_counter{
                        //print!("{}" , val.char_value as char);
                        //write!(buff_output, "{}", &output_buffer[0]);
                        buff_output.write(&[val.char_value]).expect("Could not write to output file");
                        double_break = true;
                        break;
                }
            }
            if double_break {
                break;
            }
        }
    }
    buff_output.flush().expect("Colud not flush");
    /*
    output_buffer[0] = 'x' as u8;
    output_file.write(&output_buffer).expect("Could not write to ouptut decoded file");
    */
}

fn main() {
    let huff_encoded_file_name = "encoded.huff";
    let magic_word : [char; 7] = ['H', 'U', 'F', 'F', 'M', 'A', 'N'];
    let args: Vec<_> = env::args().collect();
    if args.len() == 2{
        match args[1].trim() {
            // Match a single value
            "-t" | "--test" => {
                huff_encode("input.txt", huff_encoded_file_name, magic_word);
                huff_decode(huff_encoded_file_name, "output.txt", magic_word)
            }, 
            _ => println!("Unknown parameter"),
        }
    }
    else if args.len() == 3 {
        match args[1].trim() {
            // Match a single value
            "-e" | "--encode" => huff_encode(args[2].trim(), huff_encoded_file_name, magic_word),
            "-d" | "--decode" => huff_decode(huff_encoded_file_name, args[2].trim(), magic_word),
            _ => println!("Unknown parameter"),
        }
    }
    else{
        println!("Call with --encode 'File name' or --decode 'Encoded file name'");
    } 

}
