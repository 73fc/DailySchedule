fn main() {
    let number = 7;
    if number < 5 {
        println!("condition was true");
    } else {
        println!("condition was false");
    }
    if_else(number);
    loop_fn();
    while_fn();
    for_fn();
}

fn if_else(number: i32){
    if number % 4 == 0 {
        println!("number is divisible by 4");
    } else if number % 3 == 0 {
        println!("number is divisible by 3");
    } else if number % 2 == 0 {
        println!("number is divisible by 2");
    } else {
        println!("number is not divisible by 4,3 or 2");
    }
}

fn let_if() {
    let condition = true;
    let number = if condition {
        5
    } else {
        6 //"six"   if语句的每个分支最终的返回值必须是同一类型
    };
}


fn loop_fn() {
    let mut counter = 0;
    let result = loop {
        counter +=1;
        if counter == 10 {
            break counter *3;
        }
    };
    println!("The result is {}",result);
}

fn while_fn() {
    let mut number = 3;
    while number != 0 {
        println!("{}",number);
        number -= 1;
    }
    println!("LIFTOFF!!");
}

fn for_fn() {
    for number in (1..4).rev() { //.rev 反转遍历的方向
        println!("{}!",number);
    }
    println!("LIFTOFF!");

}

