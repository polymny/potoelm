use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Msg {
    pub comment: String,
    pub msgid: String,
    pub msgstr: Vec<String>,
    pub is_plural: bool,
}

impl Msg {
    pub fn new() -> Msg {
        Msg {
            comment: String::new(),
            msgid: String::new(),
            msgstr: vec![],
            is_plural: false,
        }
    }

    pub fn print(&self) {
        println!(
            "{}",
            self.comment
                .lines()
                .map(|x| format!("#. {}", x))
                .collect::<Vec<_>>()
                .join("\n")
        );
        println!("msgid \"{}\"", self.msgid);

        if self.msgstr.len() == 1 {
            println!("msgstr \"{}\"", self.msgstr[0].replace("\n", "\\n"));
        } else {
            println!("msgid_plural \"\"");
            for (i, msg) in self.msgstr.iter().enumerate() {
                println!("msgstr[{}] \"{}\"", i, msg.replace("\n", "\\n"));
            }
        }

        println!();
    }
}

#[derive(Debug)]
pub struct Po {
    pub lang: String,
    pub msgs: Vec<Msg>,
}

impl Po {
    pub fn new(lang: &str) -> Po {
        Po {
            lang: lang.into(),
            msgs: vec![],
        }
    }

    pub fn parse<P: AsRef<Path>>(lang: &str, p: P) -> Po {
        let mut reading_msgstr = false;
        let mut reader = BufReader::new(File::open(p.as_ref()).unwrap());
        let mut po = Po::new(lang);
        let mut msg = Msg::new();
        let mut msgstr = String::new();

        loop {
            let mut line = String::new();
            let len = reader.read_line(&mut line).unwrap();
            line.pop();

            if len == 0 {
                break;
            }

            if reading_msgstr {
                if line.starts_with("\"") {
                    msgstr.push_str(&line.split("\"").nth(1).unwrap().replace("\\n", "\n"));
                } else if line.starts_with("msgstr") {
                    msg.msgstr.push(msgstr);
                    msgstr = String::new();
                } else if line.is_empty() {
                    reading_msgstr = false;
                    msg.msgstr.push(msgstr);
                    msgstr = String::new();
                    if !msg.comment.is_empty() {
                        msg.comment.pop();
                    }
                    po.msgs.push(msg);
                    msg = Msg::new();
                    continue;
                }
            }

            if line.starts_with("#.") {
                msg.comment.push_str(&line[2..].trim());
                msg.comment.push_str("\n");
            } else if line.starts_with("msgid ") {
                msg.msgid = line.split("\"").nth(1).unwrap().to_string();
            } else if line.starts_with("msgid_plural") {
                msg.is_plural = true;
            } else if line.starts_with("msgstr") {
                reading_msgstr = true;
                msgstr = line.split("\"").nth(1).unwrap().to_string();
            }
        }

        po
    }

    pub fn print(&self) {
        for msg in &self.msgs {
            msg.print();
        }
    }
}

pub fn to_elm(pos: Vec<Po>) {
    let mut keys = HashSet::new();

    for po in &pos {
        for msgs in &po.msgs {
            keys.insert((msgs.msgid.clone(), msgs.is_plural));
        }
    }

    println!("module Strings exposing (..)");
    println!();
    println!("import Lang exposing (Lang)");
    println!();
    println!();

    for (key, is_plural) in &keys {
        if key == "" {
            continue;
        }

        let fn_name = key
            .split(".")
            .map(|x| format!("{}{}", x.chars().nth(0).unwrap().to_uppercase(), &x[1..]))
            .collect::<Vec<_>>()
            .join("");

        let fn_name = format!(
            "{}{}",
            fn_name.chars().nth(0).unwrap().to_lowercase(),
            &fn_name[1..]
        );

        print!("{} : Lang -> ", fn_name);

        if *is_plural {
            print!("Int -> ");
        }

        println!("String");

        print!("{} lang ", fn_name);
        if *is_plural {
            print!("n ");
        }
        println!("=");
        println!("    case lang of");

        for po in &pos {
            println!("        Lang.{} ->", po.lang);

            let msg = po.msgs.iter().find(|x| &x.msgid == key).unwrap();

            if !is_plural {
                println!("            \"{}\"", msg.msgstr[0]);
            } else {
                let len = msg.msgstr.len();
                for (i, msgstr) in msg.msgstr.iter().enumerate() {
                    println!(
                        "            {}",
                        if i == len - 1 {
                            "else".to_string()
                        } else {
                            format!(
                                "{}if n {} {} then",
                                if i == 0 { "" } else { "el" },
                                if i == 0 { "<=" } else { "==" },
                                i + 1
                            )
                        },
                    );
                    println!("                \"{}\"\n", msgstr);
                }
            }

            println!();
        }

        println!();
    }
}
