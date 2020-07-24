fn main() {
    println!("Hello, world!");
    let y = 6;
    another_function();
    function2(5);
    function3(3,7);
    sentence();
    let x = five();
    println!("five: x is {}",x);
    println!("plus_one: x is {}", plus_one(x));
}

/*函数的声明与调用 */
// 函数的声明方式，以及对声明位置没有要求
fn another_function() {
    println!("Another function.");
}

fn function2(x:i32) {
    println!("f2:The value of x is:{}",x);
}

fn function3(x:i32,y:i32) {
    println!("f3:The value of x is:{}",x);
    println!("f3:The value of y is:{}",y);
}

/*语句与表达式 */
fn sentence() {
    let x = 6;
    //let x = (let y = 6);  语句不返回值， 但是表达式返回值
    let y = {
        let  x = 3;
        x + 1
    };
    //println!("f3:The value of y is:{}",y);
    println!("s1: x{} y {}", x,y);
}

fn five() -> i32{
    5
}

fn plus_one(x:i32)->i32 {
    x + 1
}