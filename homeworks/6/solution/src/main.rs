fn main() {
    println!("Welcome to my Homework 6 solution!");
    let fizzbuzz_count = fizz_buzz();
    println!("{}", fizzbuzz_count)
}

fn fizz_buzz() -> u32 {
    let mut counter = 0;

    for i in 0..301 {
        let fizz = i % 3 == 0;
        let buzz = i % 5 == 0;

        match (fizz, buzz) {
            (true, false) => println!("fizz"),
            (false, true) => println!("buzz"),
            (true, true) => {
                println!("fizzbuzz");
                counter += 1;
            }
            _ => (),
        }
    }

    counter
}
