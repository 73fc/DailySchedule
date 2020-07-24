#[macro_use] extern crate lazy_static;
extern crate regex;
use regex::Regex;
const TO_SEARCH : &'static str = "On 2017-12-31, happy. On 2018-01-01, New Year.";

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?x)(?P<year>\d{4}) # 可添加注释
                                        -(?P<month>\d{2}) 
                                        -(?P<day>\d{2})").unwrap();

    static ref EMAIL_RE: Regex =  Regex::new(r"(?x)
                     ^\w+@(?:gamil|163|qq)\.(?:com|cn|com\.cn|net)$").unwrap();
}

fn regex_date(text:&str) -> regex::Captures {
    RE.captures(text).unwrap()
}

fn regex_email(text:&str) -> bool {
    EMAIL_RE.is_match(text)
}

fn main() {
    //10-7
    let re = Regex::new(r"((\d{4})-(\d{2})-(\d{2}))").unwrap();
    for caps in re.captures_iter(TO_SEARCH) {
        println!("Year:{},month:{},day:{}", caps.get(1).unwrap().as_str()
                                          , caps.get(2).unwrap().as_str()
                                          , caps.get(3).unwrap().as_str());
    }
    //10-8
    let re = Regex::new(r"(?x)(?P<year>\d{4}) # 可添加注释
                        -(?P<month>\d{2}) 
                        -(?P<day>\d{2})").unwrap();
    let caps = re.captures("2020-07-20").unwrap();
    assert_eq!("2020",&caps["year"]);
    assert_eq!("07",&caps["month"]);
    assert_eq!("20",&caps["day"]);
    let after = re.replace_all("2020-07-20","$month/$day/$year");
    assert_eq!("07/20/2020",after);

    //10-9
    let caps = regex_date("2020-07-20");
    assert_eq!("2020",&caps["year"]);
    assert_eq!("07",&caps["month"]);
    assert_eq!("20",&caps["day"]);
    let after = re.replace_all("2020-07-20","$month/$day/$year");
    assert_eq!("07/20/2020",after);
    assert_eq!(regex_email("fc7773333333@163.com"), true);
    assert_eq!(regex_email("fc7773333333@163.cn.net"), false); 
}
