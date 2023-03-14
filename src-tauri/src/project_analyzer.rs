use std::any;
use std::fmt::{Display, Formatter};
use std::io::{Write};
use std::{io::Read, error::Error};

use regex::Regex;

pub fn analyse_project(directory: &str) {
    let files = find_java_files(directory);
    
    match std::fs::metadata(&format!("{}/umlbot", directory)) {
        Ok(_) => {},
        Err(_) => std::fs::create_dir(&format!("{}/umlbot", directory)).unwrap(),
    }

    for file in files {
        let source = read_file(&file).unwrap();
        let clean_source = remove_comments(source);
        let class = Class::from_source(&clean_source).unwrap();
        println!("analysed class {}", class.class_signature.class_name);
        write_file(&format!("{}/umlbot/{}.txt", directory, file.split(|x| {x == '\\' || x == '/'}).last().unwrap().strip_suffix(".java").unwrap()), &class.to_str());
    }
}

#[derive(Debug)]
struct ParserError {}
impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Parser Error")
    }
}
impl Error for ParserError {}

#[derive(Debug, PartialEq)]
enum ClassType { Class, Interface }

#[derive(Debug)]
enum AccessType { Default, Public, Private, Protected }

struct ClassSignature {
    pub access_type: AccessType,
    pub class_type: ClassType,
    pub class_name: String,
    pub superclass_name: Option<String>,
    pub imlemented_interfaces: Vec<String>,
}
impl ClassSignature {
    fn from_source(class_source: &str) -> Result<Self, Box<dyn Error>> {
        let re = regex::Regex::new(
            r"\b(public|private)?\s*\b(class|interface)\s+(\w+)\s*(?:extends\s+(\w+)\s*)?(?:implements\s+(\w+(?:\s*,\s*\w+)*)\s*)?\{"
        ).expect("couldn't compile Regex. Exiting ...");
        
        let result = re.captures(class_source).unwrap();

        Ok(ClassSignature {
            access_type: match result.get(1) {
                None => AccessType::Default,
                Some(x) => match x.as_str() {
                    "public" => AccessType::Public,
                    "private" => AccessType::Private,
                    "protected" => AccessType::Protected,
                    _ => panic!(),
                },
            },
            class_type: match result.get(2).unwrap().as_str() {
                "class" => ClassType::Class,
                "interface" => ClassType::Interface,
                _ => panic!(),
            },
            class_name: result.get(3).unwrap().as_str().to_owned(),
            superclass_name: match result.get(4) {
                None => None,
                Some(x) => Some(x.as_str().to_owned()),
            },
            imlemented_interfaces: match result.get(5) {
                None => Vec::new(),
                Some(x) => x.as_str().to_owned().split(",").map(|s| s.trim().to_string()).collect(),
            },
        })
    }
}

struct Argument {
    pub argument_name: String,
    pub argument_type: String,
}

struct Method {
    pub access_type: AccessType,
    pub is_static: bool,
    pub return_type: Option<String>,
    pub method_name: String,
    pub arguments: Vec<Argument>,
}
impl Method {
    fn vec_from_class_source(class_source: &str) -> Result<Vec<Self>, Box<dyn Error>> {
        let re = regex::Regex::new(
            r"(;|\n)\s*(\s*(public|private|protected))?(\s*(static))?(\s*(\w+))(\s+(\w+))\((.*)\)"
        ).expect("couldn't compile Regex. Exiting ...");
        
        let mut methods = Vec::new();

        for result in re.captures_iter(class_source) {
            let mut anything_broken = false;

            let m = Method {
                access_type: match result.get(3) {
                    None => AccessType::Default,
                    Some(x) => match x.as_str() {
                        "public" => AccessType::Public,
                        "private" => AccessType::Private,
                        _ => panic!(),
                    },
                },
                is_static: match result.get(5) {
                    None => false,
                    Some(_) => true,
                },
                return_type: match result.get(7).unwrap().as_str() {
                    "void" => None,
                    x => Some(x.to_string()),
                },
                method_name: result.get(9).unwrap().as_str().to_string(),
                arguments: match result.get(10).unwrap().as_str() {
                    "" => Vec::new(),
                    x => x.split(",").filter_map(|s| {
                        let arg: Vec<&str> = s.trim().split_whitespace().collect();
                        if arg.len() != 2 {
                            anything_broken = true;
                            return None
                        }
                        Some(Argument { argument_name: arg[1].to_string(), argument_type: arg[0].to_string() })
                    }).collect(),
                },
            };

            if !anything_broken {
                match &m.return_type {
                    Some(x) => match x.as_str() {
                        "public" => {},
                        _ => methods.push(m),
                    },
                    None => methods.push(m),
                }
            }
        }

        Ok(methods)
    }
}

struct Constructor {
    pub access_type: AccessType,
    pub arguments: Vec<Argument>,
}
impl Constructor {
    fn vec_from_class_source(class_source: &str) -> Result<Vec<Self>, Box<dyn Error>> {
        let re = regex::Regex::new(
            r"(;|\n)\s*(public|private|protected)?\s*[A-Z]\w*\s*\((.*?)\)\s*(throws\s+[A-Za-z0-9]+\s*)?(\{)?"
        ).expect("couldn't compile Regex. Exiting ...");
        
        let mut constructors = Vec::new();

        for result in re.captures_iter(class_source) {
            constructors.push(Constructor {
                access_type: match result.get(2) {
                    None => AccessType::Default,
                    Some(x) => match x.as_str() {
                        "public" => AccessType::Public,
                        "private" => AccessType::Private,
                        _ => AccessType::Default,
                    },
                },
                arguments: match result.get(3).unwrap().as_str() {
                    "" => Vec::new(),
                    x => x.split(",").map(|s| {
                        let arg: Vec<&str> = s.trim().split_whitespace().collect();
                        Argument { argument_name: arg[1].to_string(), argument_type: arg[0].to_string() }
                    }).collect(),
                },
            });
        }

        Ok(constructors)
    }
}

struct Member {
    pub access_type: AccessType,
    pub member_name: String,
    pub member_type: String,
}
impl Member {
    fn vec_from_class_source(class_source: &str) -> Result<Vec<Member>, Box<dyn Error>> {
        let re = regex::Regex::new(
            r"(public|protected|private)\s+(\w+)\s+(\w+)\s*;"
        ).expect("couldn't compile Regex. Exiting ...");
        
        let mut members = Vec::new();

        for result in re.captures_iter(class_source) {
            members.push(Member {
                access_type: match result.get(1).unwrap().as_str() {
                    "public" => AccessType::Public,
                    "private" => AccessType::Private,
                    "protected" => AccessType::Protected,
                    _ => panic!(),
                },
                member_name: result.get(3).unwrap().as_str().to_string(),
                member_type: result.get(2).unwrap().as_str().to_string(),
            })
        }

        Ok(members)
    }
}

struct Class {
    class_signature: ClassSignature,
    constructors: Vec<Constructor>,
    methods: Vec<Method>,
    members: Vec<Member>,
}
impl Class {
    fn from_source(class_source: &str) -> Result<Self, Box<dyn Error>> {
        // TODO: dd initializers struct to classes
        Ok(Class {
            class_signature: ClassSignature::from_source(class_source)?,
            constructors: Constructor::vec_from_class_source(class_source)?,
            methods: Method::vec_from_class_source(class_source)?,
            members: Member::vec_from_class_source(class_source)?,
        })
    }
    
    fn print(&self) {
        println!("{}", self.to_str());
    }

    fn to_str(&self) -> String {
        let mut str = String::new();

        str += &format!("{}\n", match &self.class_signature.access_type {
            AccessType::Default => "",
            AccessType::Public => "public",
            AccessType::Private => "private",
            AccessType::Protected => "protected",
        });

        if self.class_signature.class_type == ClassType::Interface { str += &format!("<<interface>>\n"); }

        str += &format!("{}\n", self.class_signature.class_name);

        match &self.class_signature.superclass_name {
            Some(x) => str += &format!("extends {}\n", x),
            None => {},
        }

        for interface in &self.class_signature.imlemented_interfaces {
            str += &format!("implements {}\n", interface);
        }
        for member in self.members.iter() {
            str += &format!("{} {}: {}\n",
                match member.access_type {
                    AccessType::Default => " ",
                    AccessType::Public => "+",
                    AccessType::Private => "-",
                    _ => panic!(),
                },
                member.member_name,
                member.member_type,
            );
        }
        for constructor in self.constructors.iter() {
            str += &format!("{} {}({})\n",
                match constructor.access_type {
                    AccessType::Default => " ",
                    AccessType::Public => "+",
                    AccessType::Private => "-",
                    AccessType::Protected => "#",
                },
                self.class_signature.class_name,
                constructor.arguments.iter().map(|x| format!("{}: {}", x.argument_name, x.argument_type)).collect::<Vec<String>>().join(", ").to_string(),
            );
        }
        for method in self.methods.iter() {
            str += &format!("{} {}({}): {}{}\n",
                match method.access_type {
                    AccessType::Default => " ",
                    AccessType::Public => "+",
                    AccessType::Private => "-",
                    AccessType::Protected => "#",
                },
                method.method_name,
                method.arguments.iter().map(|x| format!("{}: {}", x.argument_name, x.argument_type)).collect::<Vec<String>>().join(", ").to_string(),
                match &method.return_type {
                    None => "void",
                    Some(x) => &x,
                },
                match &method.is_static {
                    true => " [static]",
                    false => "",
                }
            );
        }

        str
    }
}

fn read_file(path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = std::fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn find_java_files(dir: &str) -> Vec<String> {
    println!("searching {}", dir);

    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap() {
        match entry {
            Ok(file) => {
                let path = file.path().display().to_string();
                if path.ends_with(".java") {
                    println!("found {}", path);
                    files.push(path);
                }
            },
            Err(_) => {},
        }
    }
    
    files
}

fn write_file(filename: &str, content: &str) {
    match std::fs::File::create(filename) {
        Ok(mut file) => { file.write_all(content.as_bytes()).unwrap(); },
        Err(_) => println!("couldn't create or open file {}", filename),
    }
}

fn remove_comments(source: String) -> String {
    let re = Regex::new(r"(?:\\/\\*(?:[\\s\\S]*?)\\*\\/)|(?:\\/\\/.*)").expect("couldn't compile Regex. Exiting ...");
    re.replace_all(&source, "").to_string()
}
