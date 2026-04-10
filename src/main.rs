//! Malvin — application entry point.

fn main() {
    println!("Hello, malvin!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn main_smoke() {
        super::main();
    }
}
