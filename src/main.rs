use std::{
    env,
    fmt::{Display, Write},
};

#[derive(PartialEq, Debug)]
#[allow(dead_code)]
enum Bencode {
    String(String),
    Integer(i64),
    List(Vec<Bencode>),
}

impl Display for Bencode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bencode::String(s) => f.write_str(format!(r#""{s}""#).as_str()),
            Bencode::Integer(i) => f.write_str(format!("{i}").as_str()),
            Bencode::List(l) => {
                f.write_char('[').unwrap();

                for (i, bencode) in l.iter().enumerate() {
                    f.write_str(format!("{bencode}").as_str())?;
                    if i + 1 < l.len() {
                        f.write_str(", ")?;
                    }
                }

                f.write_char(']')
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
            let mut list_string = encoded_value
                .strip_prefix('l')
                .unwrap()
                .strip_suffix('e')
                .unwrap();

            let mut list = Vec::new();

            loop {
                let (decoded_value, rest) = decode_bencoded_value(list_string);
                list.push(decoded_value);
                if rest == "" {
                    break;
                };

                list_string = rest
            }

            return (Bencode::List(list), "");
        }
        _ => panic!("Unhandled encoded value: {}", encoded_value),
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
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
}
