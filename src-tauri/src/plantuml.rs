use std::{process::Command};
use tauri::Manager;

use crate::project_analyzer::Class;

pub fn classes_to_graph(classes: &Vec<Class>, directory: &str, custom_java: Option<&str>, app: &tauri::AppHandle) {
    std::fs::create_dir(format!("{}/umlbot", directory)).ok();

    println!("{:#?}", custom_java);

    for class in classes.iter() {
        app.emit_all("analysis_info", format!("Exporting {}", class.class_signature.class_name)).unwrap();
        let mut uml = String::new();
    
        uml.push_str("@startuml\n");
        uml.push_str(&format!("class {} {{\n", class.class_signature.class_name));
        for member in class.members.iter() {
            uml.push_str(&format!("{} {}\n", member.member_type, member.member_name));
        }
        for method in class.methods.iter() {
            uml.push_str(&format!("{} {}(",
            match &method.return_type {
                Some(x) => x,
                None => "void",
            },
            method.method_name));

            uml.push_str(&method.arguments.iter().map(|x| format!("{}: {}", x.argument_name, x.argument_type)).collect::<Vec<String>>().join(", "));

            uml.push_str(")\n");
        }
        uml.push_str("}\n");
        uml.push_str("@enduml\n");

        std::fs::write(format!("{}/umlbot/temp.txt", directory), uml).expect("failed to write to file");

        Command::new(
            match custom_java {
                None => "java",
                Some(x) => x,
            }
        )
            .args(["-jar", "plantuml.jar", "-I", &format!("{}/umlbot/temp.txt", directory), "-tsvg"])
            .spawn().unwrap().wait().unwrap();

        std::fs::rename(&format!("{}/umlbot/temp.svg", directory), &format!("{}/umlbot/{}.svg", directory, class.class_signature.class_name)).unwrap();
        std::fs::remove_file(&format!("{}/umlbot/temp.txt", directory)).unwrap();
    }
}
