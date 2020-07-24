
fn main() {
    //可变变量
    let mut x = 5;
    println!("The value of x is :{}",x);
    x = 6; 
    println!("The value of x is:{}",x);

    //常量
    const MAX_POINTS: u32 = 100_000;

    //隐藏
    let y = 5;
    let y = y + 3;
    let y = y + 7;
    println!("The value of y is :{}", y);

    let spaces = "   ";
    let spaces = spaces.len(); //正确用法
    /*错误原因： spaces是可变变量， 但是数据类型是确定的，不能随意更换类型
    let mut spaces = "   ";
    spaces = spaces.len();  
    */
    

    //当数据继续转换时，必须要有确定的赋值类型， 简单来讲就是任何语句和数据的结果都必须“确定”
    let guess: u32 = "42".parse().expect("Not a number!"); 
    
    //浮点
    let d = 2.0; //默认f64, 速度与f32相差无几
    let f : f32 = 3.0; //f32需要指定
    

    //数据运算
    let sum = 5 + 10;
    let difference = 95.5 - 4.3;
    let product = 4 * 30;
    let quotient = 56.7 / 32.2;
    let remainder  = 43 % 5;

    //布尔
    let t = true;
    let f: bool = false;

    //字符类型： 注意大小为四个字节，用unicode编码，包括了各种字符和符号
    let c = 'Z';
    let z = 'z';
    let cat = '🐱';
    println!("cat {}",cat);


    /* 复合类型： 1.元组 2. 数组 */
    //元组
    let tup:(i32, f64,u8) = (500,3.7,3);
    let (x,y,z) = tup;      // 解构
    let (x,y,z) = (500,3.7,3); // 疑问？ 此时未指定数据类型， 存有不确定性，为何能通过？，都是默认？
    let five_hundred = tup.0;
    let three_point_seven = tup.1;
    let three = tup.2;
    
    // 数组: 固定长度，在栈上
    let a = [1,2,3,4,5];
    let a = [3;5]; // 5个元素，每个元素都是3
    let a : [i32;5] = [5,4,3,2,1]; // 长度为5，类型均为i32
    let first = a[0]; // 下标从0开始，访问数组元素
    let second = a[1];
    /*无法在编译时检测出来的下标越界问题，  rust用panic处理。
    let index = 10;
    println!("The value of element is:{}",a[index]);
    */
}


