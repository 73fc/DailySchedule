use std::io;

fn main() {
    println!("Hello, world!");
    homework1();
    let n = 3;
    let fibn = homework2(n);
    println!("fib[{}] = {}",n,fibn);
    homework3();
}

fn homework1() {
    println!("please input the degree centigrade");
    let mut degree = String::new();
    io::stdin().read_line(&mut degree)
    .expect("Failed to read line");
    let degree :f64 = match degree.trim().parse() {
        Ok(num) => num,
        Err(_) => 0.0,
    };
    let fahrenheit = degree * 3.88;
    println!("degree centigrade:{}, fahrenheit:{}", degree, fahrenheit);
}

fn homework2(n:i32) -> i32 {
    if n == 1 || n == 0 {
        return 1;
    } else {
        let mut x = 1;
        let mut y = 2;
        for _ in (2..n) {
            y = y + x;
            x = y - x;    
        }
        return y;
    }
}

fn homework3() {
    println!("The Twelve days of Christmas.");
    for i in (1..13) {
        print!("On the {} day of Christmas, my true love sent to me:",i);
        if 11 < i { print!("Twelve drummers drumming,"); }
        if 10 < i { print!("Eleven pipers piping,"); }
        if 9 < i { print!("Ten lords a-leaping,"); }
        if 8 < i { print!("Nine ladies dancing,"); }
        if 7 < i { print!("Eight maids a-milking, "); }
        if 6 < i { print!("Seven swans a-swimming,"); }
        if 5 < i { print!("Six geese a-laying,"); }
        if 4 < i { print!("Five golden rings,"); }
        if 3 < i { print!("Four calling birds,"); }
        if 2 < i { print!("Three French hens,"); }
        if 1 < i { print!("Two turtle doves, And "); }
        println!("A partridge in a pear tree.");
    }
}