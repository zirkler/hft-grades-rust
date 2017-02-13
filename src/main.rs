extern crate hyper;
extern crate rustc_serialize;
extern crate url;
extern crate hyper_native_tls;
extern crate xml;

use std::io;
use std::io::Read;
use url::form_urlencoded;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper::header::ContentType;
use xml::{Event};

struct Grade {
    course: String,
    course_id: u32,
    grade_str: String,
    passed: String,
    grade: f64,
    ects: f64,
}

fn main() {

    println!("{:?}", "Enter Username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed somehow :(");

    println!("{:?}", "Enter Password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password).expect("Failed somehow :(");

    let url = "https://php.rz.hft-stuttgart.de/hftapp/notenhftapp.php";
    let query = vec![("username", username.trim()), ("password", password.trim())];

    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(query.iter())
        .finish();
    let mut response = client.post(url)
        .header(ContentType("application/x-www-form-urlencoded".parse().unwrap()))
        .body(&body[..])
        .send()
        .unwrap();
    let mut response_str = String::new();
    response.read_to_string(&mut response_str).unwrap();

    let mut grades = Vec::new();
    let mut current_element = String::new();
    let mut p = xml::Parser::new();
    p.feed_str(&response_str);
    
    for event in p {
        match event.unwrap() {
            Event::ElementStart(tag) => {
                if tag.name == "e" {
                    let grade = Grade {
                        course: String::new(),
                        course_id: 0,
                        passed: String::new(),
                        grade_str: String::new(),
                        grade: 0.0,
                        ects: 0.0
                    };
                    grades.push(grade);
                }
                current_element = tag.name
            },
            Event::Characters(s) => {
                if !s.trim().is_empty() {
                    let curr_grade = grades.last_mut().unwrap();
                    match current_element.as_ref() {
                        "text" => {
                            curr_grade.course = s;
                        },
                        "note" => {
                            curr_grade.grade_str = s;
                            curr_grade.grade = curr_grade.grade_str.parse().unwrap();
                        },
                        "bonus" => {
                            curr_grade.ects = s.parse().unwrap();
                        },
                        "nummer" => {
                            curr_grade.course_id = s.parse().unwrap();
                        },
                        "bestanden" => {
                            curr_grade.passed = s;
                        },
                        _ => { },
                    }
                }
            },
            _ => ()
        }
    }

    grades.retain(|g| g.ects > 0.0);
    grades.retain(|g| g.course_id > 2999);
    grades.retain(|g| g.passed == "BE");
    grades.retain(|g| g.grade > 0.0);

    let mut cum_grades:f64 = 0.0;
    let mut total_ects:f64 = 0.0;
    for grade in &grades {
        cum_grades += (grade.grade as f64) * grade.ects;
        total_ects += grade.ects; 
        println!("Note {:.1} ({:?} ECTS) in {:?}", grade.grade/100.0, grade.ects, grade.course);
    }

    println!("Num of Grades: {:?}", grades.len());
    println!("==> {:.2}", cum_grades/100.0/total_ects);
}