use std::{
    collections::HashMap,
    env,
    fmt::{Display, Write},
};

#[derive(PartialEq, Debug)]
#[allow(dead_code)]
enum Bencode {
    String(String),
    Integer(i64),
    List(Vec<Bencode>),
    Dictionary(HashMap<String, Bencode>),
}

impl Display for Bencode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bencode::String(s) => f.write_str(format!(r#""{s}""#).as_str()),
            Bencode::Integer(i) => f.write_str(format!("{i}").as_str()),
            Bencode::List(l) => {
                f.write_char('[')?;

                for (i, bencode) in l.iter().enumerate() {
                    f.write_str(format!("{bencode}").as_str())?;
                    if i + 1 < l.len() {
                        f.write_str(", ")?;
                    }
                }

                f.write_char(']')
            }
            Bencode::Dictionary(d) => {
                f.write_char('{')?;

                for (i, (key, value)) in d.iter().enumerate() {
                    f.write_str(format!(r#""{key}": {value}"#).as_str())?;
                    if i + 1 < d.len() {
                        f.write_str(", ")?;
                    }
                }

                f.write_char('}')
            }
        }
    }
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (Bencode, &str) {
    // If encoded_value starts with a digit, it's a number
    match encoded_value.chars().next().unwrap() {
        '0'..='9' => {
            if let Some((len, rest)) = encoded_value.split_once(":") {
                if let Ok(len) = len.parse::<usize>() {
                    return (Bencode::String(rest[..len].to_string()), &rest[len..]);
                }
            }

            panic!("Error decoding Bencode string")
        }
        'i' => {
            let (number_string, rest) = encoded_value
                .strip_prefix('i')
                .unwrap()
                .split_once('e')
                .unwrap();
            if number_string.chars().next().unwrap() == '0' && number_string.len() > 1 {
                panic!("All encodings with a leading zero are invalid, other than i0e")
            }

            if number_string == "-0" {
                panic!("i-0e is invalid")
            }

            let number = number_string.parse::<i64>().unwrap();
            return (Bencode::Integer(number), rest);
        }
        'l' => {
            let mut list_string = encoded_value.strip_prefix('l').unwrap();

            let mut list = Vec::new();

            loop {
                let (decoded_value, rest) = decode_bencoded_value(list_string);
                list.push(decoded_value);
                if rest.chars().next().unwrap() == 'e' {
                    return (Bencode::List(list), rest.strip_prefix('e').unwrap());
                };

                list_string = rest
            }
        }
        'd' => {
            let mut dict_string = encoded_value.strip_prefix('d').unwrap();

            let mut dict = HashMap::new();

            while let (Bencode::String(key), rest) = decode_bencoded_value(dict_string) {
                let (value, rest) = decode_bencoded_value(rest);
                dict.insert(key, value);
                if rest.chars().next().unwrap() == 'e' {
                    return (Bencode::Dictionary(dict), rest.strip_prefix('e').unwrap());
                };

                dict_string = rest
            }

            return (Bencode::Dictionary(dict), "");
        }
        _ => panic!("Unhandled encoded value: {}", encoded_value),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.0.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_bencode_sting() {
        assert_eq!(
            decode_bencoded_value("3:Hey"),
            (Bencode::String("Hey".to_string()), "")
        );
        assert_eq!(
            decode_bencoded_value("4:Test"),
            (Bencode::String("Test".to_string()), "")
        )
    }

    #[test]
    fn decode_bencode_integer() {
        assert_eq!(decode_bencoded_value("i30e"), (Bencode::Integer(30), ""));
        assert_eq!(decode_bencoded_value("i-42e"), (Bencode::Integer(-42), ""));
    }

    #[test]
    fn decode_bencode_list() {
        assert_eq!(
            decode_bencoded_value("l4:spam4:eggse"),
            (
                Bencode::List(vec![
                    Bencode::String("spam".to_string()),
                    Bencode::String("eggs".to_string())
                ]),
                ""
            )
        );
        assert_eq!(
            decode_bencoded_value("l5:helloi52ee"),
            (
                Bencode::List(vec![
                    Bencode::String("hello".to_string()),
                    Bencode::Integer(52)
                ]),
                ""
            )
        )
    }

    #[test]
    fn decode_bencode_dictionary() {
        let mut test = HashMap::new();
        test.insert("foo".to_string(), Bencode::String("bar".to_string()));
        test.insert("hello".to_string(), Bencode::Integer(52));

        assert_eq!(
            decode_bencoded_value("d3:foo3:bar5:helloi52ee"),
            (Bencode::Dictionary(test), "")
        )
    }
}
