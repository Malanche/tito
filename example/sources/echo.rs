fn main() {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s);
    println!("{}!", s.trim());
}