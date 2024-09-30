use std::{f64, io};

fn main() {
    let mut current_sum: f64 = 0.0;
    let mut first_operation = true;

    loop {
        println!("Оберіть операцію: +, -, /, *, c, off");

        let choice = get_input_str().trim().to_lowercase();

        match choice.as_str() {
            "+" | "-" | "/" | "*" => {
                if first_operation {
                    current_sum = get_input_number("Введіть початкове значення:");
                    first_operation = false;
                }

                let next_number = get_input_number("Введіть наступне значення:");
                current_sum = perform_operation(&choice, current_sum, next_number);

                println!("Поточне значення: {}", current_sum);
            }
            "c" => {
                current_sum = 0.0;
                first_operation = true;
                println!("Суму анульовано.");
            }
            "off" => {
                println!("Вихід...");
                break;
            }
            _ => {
                println!("Помилка введення дії, оберіть з: +, -, /, *, c, off.");
            }
        }
    }
}

fn get_input_str() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Помилка при зчитанні данних");
    input
}

fn get_input_number(prompt: &str) -> f64 {
    loop {
        println!("{}", prompt);
        let input = get_input_str().trim().parse::<f64>();

        match input {
            Ok(num) => return num,
            Err(_) => {
                println!("Помилка введення, змінна має бути числом!");
            }
        }
    }
}

fn perform_operation(operation: &str, current_sum: f64, next_number: f64) -> f64 {
    match operation {
        "+" => {
            println!("Додавання...");
            current_sum + next_number
        }
        "-" => {
            println!("Віднімання...");
            current_sum - next_number
        }
        "/" => {
            if next_number == 0.0 {
                println!("Помилка! Ділення на нуль!");
                return current_sum;
            }
            println!("Ділення...");
            current_sum / next_number
        }
        "*" => {
            println!("Множення...");
            current_sum * next_number
        }
        _ => unreachable!(),
    }
}
