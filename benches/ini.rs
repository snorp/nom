#![feature(test)]
extern crate test;

#[macro_use]
extern crate nom;

use nom::{alphanumeric, multispace, space, IResult};

use std::str;
use std::collections::HashMap;

named!(
  category<&str>,
  map_res!(
    delimited!(
      char!('['),
      take_while!(call!(|c| c != ']' as u8)),
      char!(']')
    ),
    str::from_utf8
  )
);

named!(key_value    <&[u8],(&str,&str)>,
  do_parse!(
     key: map_res!(alphanumeric, str::from_utf8)
  >>      opt!(space)
  >>      char!('=')
  >>      opt!(space)
  >> val: map_res!(
           take_while!(call!(|c| c != '\n' as u8 && c != ';' as u8)),
           str::from_utf8
         )
  >>      opt!(pair!(char!(';'), take_while!(call!(|c| c != '\n' as u8))))
  >>      (key, val)
  )
);

named!(keys_and_values<&[u8], HashMap<&str, &str> >,
  map!(
    many0!(terminated!(key_value, opt!(multispace))),
    |vec: Vec<_>| vec.into_iter().collect()
  )
);

named!(category_and_keys<&[u8],(&str,HashMap<&str,&str>)>,
  do_parse!(
    category: category         >>
              opt!(multispace) >>
    keys: keys_and_values      >>
    (category, keys)
  )
);

named!(categories<&[u8], HashMap<&str, HashMap<&str,&str> > >,
  map!(
    many0!(
      separated_pair!(
        category,
        opt!(multispace),
        map!(
          many0!(terminated!(key_value, opt!(multispace))),
          |vec: Vec<_>| vec.into_iter().collect()
        )
      )
    ),
    |vec: Vec<_>| vec.into_iter().collect()
  )
);

/*
#[test]
fn parse_category_test() {
  let ini_file = &b"[category]

parameter=value
key = value2"[..];

  let ini_without_category = &b"parameter=value
key = value2"[..];

  let res = category(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_category, "category"));
}

#[test]
fn parse_key_value_test() {
  let ini_file = &b"parameter=value
key = value2"[..];

  let ini_without_key_value = &b"key = value2"[..];

  let res = key_value(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, (o1, o2)) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_key_value, ("parameter", "value")));
}


#[test]
fn parse_key_value_with_space_test() {
  let ini_file = &b"parameter = value
key = value2"[..];

  let ini_without_key_value = &b"key = value2"[..];

  let res = key_value(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, (o1, o2)) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_key_value, ("parameter", "value")));
}

#[test]
fn parse_key_value_with_comment_test() {
  let ini_file = &b"parameter=value;abc
key = value2"[..];

  let ini_without_key_value = &b"key = value2"[..];

  let res = key_value(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, (o1, o2)) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_key_value, ("parameter", "value")));
}

#[test]
fn parse_multiple_keys_and_values_test() {
  let ini_file = &b"parameter=value;abc

key = value2

[category]"[..];

  let ini_without_key_value = &b"[category]"[..];

  let res = keys_and_values(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, ref o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  let mut expected: HashMap<&str, &str> = HashMap::new();
  expected.insert("parameter", "value");
  expected.insert("key", "value2");
  assert_eq!(res, IResult::Done(ini_without_key_value, expected));
}

#[test]
fn parse_category_then_multiple_keys_and_values_test() {
  //FIXME: there can be an empty line or a comment line after a category
  let ini_file = &b"[abcd]
parameter=value;abc

key = value2

[category]"[..];

  let ini_after_parser = &b"[category]"[..];

  let res = category_and_keys(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, ref o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  let mut expected_h: HashMap<&str, &str> = HashMap::new();
  expected_h.insert("parameter", "value");
  expected_h.insert("key", "value2");
  assert_eq!(res, IResult::Done(ini_after_parser, ("abcd", expected_h)));
}

#[test]
fn parse_multiple_categories_test() {
  let ini_file = &b"[abcd]

parameter=value;abc

key = value2

[category]
parameter3=value3
key4 = value4
\0"[..];

  let ini_after_parser = &b"\0"[..];

  let res = categories(ini_file);
  //println!("{:?}", res);
  match res {
    IResult::Done(i, ref o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  let mut expected_1: HashMap<&str, &str> = HashMap::new();
  expected_1.insert("parameter", "value");
  expected_1.insert("key", "value2");
  let mut expected_2: HashMap<&str, &str> = HashMap::new();
  expected_2.insert("parameter3", "value3");
  expected_2.insert("key4", "value4");
  let mut expected_h: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
  expected_h.insert("abcd",     expected_1);
  expected_h.insert("category", expected_2);
  assert_eq!(res, IResult::Done(ini_after_parser, expected_h));
}
*/

#[bench]
fn bench_ini(b: &mut test::Bencher) {
  let str = "[owner]
name=John Doe
organization=Acme Widgets Inc.

[database]
server=192.0.2.62
port=143
file=payroll.dat
\0";

  b.iter(|| categories(str.as_bytes()).unwrap());
  b.bytes = str.len() as u64;
}

#[bench]
fn bench_ini_keys_and_values(b: &mut test::Bencher) {
  let str = "server=192.0.2.62
port=143
file=payroll.dat
\0";

  named!(acc<Vec<(&str, &str)>>, many0!(key_value));

  b.iter(|| acc(str.as_bytes()).unwrap());
  b.bytes = str.len() as u64;
}

#[bench]
fn bench_ini_key_value(b: &mut test::Bencher) {
  let str = "server=192.0.2.62\n";

  b.iter(|| key_value(str.as_bytes()).unwrap());
  b.bytes = str.len() as u64;
}
